use crate::{TestCli, TestDataset, TestSource};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

/// Three comments: an old un-annotated one (pruned), an old annotated one (kept —
/// annotated comments are never deleted), and a recent one (kept — after cutoff).
const COMMENTS: &str = concat!(
    r#"{"comment":{"id":"old-plain","timestamp":"2020-01-01T00:00:00Z","messages":[{"body":{"text":"old plain"}}]}}"#,
    "\n",
    r#"{"comment":{"id":"old-labelled","timestamp":"2020-01-02T00:00:00Z","messages":[{"body":{"text":"old labelled"}}]},"labelling":[{"group":"default","assigned":[{"name":"A","sentiment":"positive"}]}]}"#,
    "\n",
    r#"{"comment":{"id":"recent","timestamp":"2030-01-01T00:00:00Z","messages":[{"body":{"text":"recent"}}]}}"#,
);

// The fixture's old data is dated 2020 and its recent data 2030, so a one-year
// cutoff (now - 365 days) always falls between them.
const OLDER_THAN_DAYS: &str = "365";

fn email_jsonl(id: &str, mailbox: &str, timestamp: &str) -> String {
    serde_json::json!({
        "id": id,
        "mailbox": mailbox,
        "timestamp": timestamp,
        "mime_content": format!(
            "Date: {timestamp}\r\nFrom: {mailbox}\r\nTo: support@reinfer.io\r\n\
             Subject: {id}\r\nContent-Type: text/plain\r\n\r\nHello from {id}\r\n"
        ),
    })
    .to_string()
}

/// An old comment carrying (or omitting) a "Mailbox ID" string user property — the
/// property parse-email writes the originating mailbox into. `labelled` makes it
/// reviewed (so it's only deleted under --include-annotated).
fn mailbox_comment(id: &str, mailbox: Option<&str>, labelled: bool) -> String {
    let user_properties = match mailbox {
        Some(mailbox) => serde_json::json!({ "string:Mailbox ID": mailbox }),
        None => serde_json::json!({}),
    };
    let mut record = serde_json::json!({
        "comment": {
            "id": id,
            "timestamp": "2020-01-01T00:00:00Z",
            "messages": [{"body": {"text": id}}],
            "user_properties": user_properties,
        }
    });
    if labelled {
        record["labelling"] = serde_json::json!([{"group": "default", "assigned": [{"name": "A", "sentiment": "positive"}]}]);
    }
    record.to_string()
}

/// An old comment whose only annotation is a *dismissed* label (no assigned label).
/// It is still reviewed, so prune must keep it in default mode.
fn dismissed_comment(id: &str) -> String {
    serde_json::json!({
        "comment": {
            "id": id,
            "timestamp": "2020-01-01T00:00:00Z",
            "messages": [{"body": {"text": id}}],
        },
        "labelling": [{"group": "default", "dismissed": [{"name": "A", "sentiment": "positive"}]}],
    })
    .to_string()
}

/// Number of non-empty JSONL lines an output / file contains.
fn jsonl_count(text: &str) -> usize {
    text.lines().filter(|line| !line.trim().is_empty()).count()
}

/// A prune `--backup-dir` creates exactly one run-stamped subdirectory; return it.
fn run_dir(backup_parent: &Path) -> PathBuf {
    let mut entries: Vec<PathBuf> = fs::read_dir(backup_parent)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "expected one run dir in {backup_parent:?}"
    );
    entries.pop().unwrap()
}

/// Each per-resource backup subdirectory holds exactly one file (one source /
/// one bucket in these tests); return its contents.
fn only_backup_file(dir: &Path) -> String {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(entries.len(), 1, "expected one backup file in {dir:?}");
    fs::read_to_string(entries.pop().unwrap()).unwrap()
}

/// Contents of every backup file in a subdirectory, concatenated — for the
/// multi-source / multi-bucket runs where a phase writes more than one file.
fn all_backup_files(dir: &Path) -> String {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("")
}

/// Number of files in a backup subdirectory (one per source / bucket / dataset).
fn backup_file_count(dir: &Path) -> usize {
    fs::read_dir(dir).unwrap().count()
}

/// Paths of every annotation backup file. Annotations are laid out per
/// `(dataset, source)` — `annotations/<dataset-id>/<source-id>.jsonl` — so this
/// descends one level into the per-dataset directories.
fn annotation_files(run_dir: &Path) -> Vec<PathBuf> {
    let mut dataset_dirs: Vec<PathBuf> = fs::read_dir(run_dir.join("annotations"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    dataset_dirs.sort();
    let mut files = Vec::new();
    for dataset_dir in dataset_dirs {
        let mut entries: Vec<PathBuf> = fs::read_dir(&dataset_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        entries.sort();
        files.extend(entries);
    }
    files
}

/// Contents of every annotation backup file, concatenated.
fn all_annotations(run_dir: &Path) -> String {
    annotation_files(run_dir)
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("")
}

struct Fixture {
    bucket: String,
    // Held in `Option` so `Drop` can delete the source (and dataset) before the
    // bucket — a bucket with an active source attached can't be deleted.
    source: Option<TestSource>,
    dataset: Option<TestDataset>,
}

impl Fixture {
    /// A bucket of emails, a source pulling from it with comments, and a dataset
    /// containing the source with one annotated comment.
    fn new() -> Self {
        let cli = TestCli::get();
        let owner = TestCli::project();

        let bucket = format!("{owner}/test-prune-{}", Uuid::new_v4());
        cli.run(["create", "bucket", &bucket]);

        let source = TestSource::new_args(&["--bucket", &bucket]);
        let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);

        cli.run_with_stdin(
            [
                "create",
                "comments",
                "--allow-duplicates",
                "--yes",
                &format!("--source={}", source.identifier()),
                &format!("--dataset={}", dataset.identifier()),
            ],
            COMMENTS.as_bytes(),
        );

        let emails = [
            email_jsonl("e-old", "alice@reinfer.io", "2020-01-01T00:00:00Z"),
            email_jsonl("e-recent", "alice@reinfer.io", "2030-01-01T00:00:00Z"),
        ]
        .join("\n");
        cli.run_with_stdin(["create", "emails", "-y", "-b", &bucket], emails.as_bytes());

        Self {
            bucket,
            source: Some(source),
            dataset: Some(dataset),
        }
    }

    fn source(&self) -> &TestSource {
        self.source.as_ref().unwrap()
    }

    fn dataset(&self) -> &TestDataset {
        self.dataset.as_ref().unwrap()
    }

    fn comment_count(&self) -> usize {
        let cli = TestCli::get();
        jsonl_count(&cli.run(["get", "comments", self.source().identifier()]))
    }

    fn email_count(&self) -> usize {
        let cli = TestCli::get();
        jsonl_count(&cli.run(["get", "emails", &self.bucket]))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Delete the dataset and source first (via their own Drop impls), then the
        // bucket, which can't be deleted while the source is attached. Best-effort
        // so a failing test surfaces its real error rather than a destructor panic.
        self.dataset.take();
        self.source.take();
        let _ = TestCli::get().run_and_result(["delete", "bucket", &self.bucket]);
    }
}

fn temp_backup_dir() -> PathBuf {
    std::env::temp_dir().join(format!("re-prune-test-{}", Uuid::new_v4()))
}

#[test]
fn test_prune_dry_run_backs_up_without_deleting() {
    let cli = TestCli::get();
    let fixture = Fixture::new();
    let backup_parent = temp_backup_dir();

    cli.run([
        "prune",
        "--datasets",
        fixture.dataset().identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--dry-run",
        "--no-progress",
    ]);

    // Nothing deleted.
    assert_eq!(
        fixture.comment_count(),
        3,
        "dry run must not delete comments"
    );
    assert_eq!(fixture.email_count(), 2, "dry run must not delete emails");

    // But backups were written: only the old, un-annotated comment and the old
    // email are in the deletion set; the annotated comment is in the annotations.
    let dir = run_dir(&backup_parent);
    assert_eq!(
        jsonl_count(&only_backup_file(&dir.join("deleted-comments"))),
        1
    );
    assert_eq!(
        jsonl_count(&only_backup_file(&dir.join("deleted-emails"))),
        1
    );
    assert!(only_backup_file(&dir.join("deleted-comments")).contains("old-plain"));
    assert!(only_backup_file(&dir.join("deleted-emails")).contains("e-old"));
    assert!(dir.join("manifest.json").exists());

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_and_restore() {
    let cli = TestCli::get();
    let fixture = Fixture::new();
    let backup_parent = temp_backup_dir();

    // Prune everything dated before the cutoff.
    cli.run([
        "prune",
        "--datasets",
        fixture.dataset().identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // Old un-annotated comment + old email are gone; annotated and recent remain.
    assert_eq!(
        fixture.comment_count(),
        2,
        "old un-annotated comment pruned"
    );
    assert_eq!(fixture.email_count(), 1, "old email pruned");

    let dir = run_dir(&backup_parent);
    let comment_backup = only_backup_file(&dir.join("deleted-comments"));
    let email_backup = only_backup_file(&dir.join("deleted-emails"));
    assert!(comment_backup.contains("old-plain"));
    assert!(
        !comment_backup.contains("old-labelled"),
        "annotated kept out of deletion set"
    );
    assert!(email_backup.contains("e-old"));

    // The annotated comment is preserved in the annotations backup, never the
    // deletion set — this is the safety guarantee the whole command exists for.
    let annotation_backup = all_annotations(&dir);
    assert!(
        annotation_backup.contains("old-labelled"),
        "annotated comment backed up under annotations/"
    );

    // Restore straight from the backups using the `re create` commands — this is
    // the contract: backups are written in the formats `re create` expects.
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--yes",
            "--allow-duplicates",
            &format!("--source={}", fixture.source().identifier()),
        ],
        comment_backup.as_bytes(),
    );
    cli.run_with_stdin(
        ["create", "emails", "-y", "-b", &fixture.bucket],
        email_backup.as_bytes(),
    );

    assert_eq!(fixture.comment_count(), 3, "comment restored from backup");
    assert_eq!(fixture.email_count(), 2, "email restored from backup");

    // Content fidelity, not just count: the restored records carry their original
    // payload. `contains` (not equality) tolerates the backend re-wrapping MIME headers.
    let restored_comments = cli.run(["get", "comments", fixture.source().identifier()]);
    assert!(
        restored_comments.contains("old plain"),
        "restored comment body preserved: {restored_comments}"
    );
    let restored_emails = cli.run(["get", "emails", &fixture.bucket]);
    assert!(
        restored_emails.contains("Hello from e-old"),
        "restored email mime content preserved: {restored_emails}"
    );

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_mailbox_filters_comments_by_user_property() {
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    // Three old, un-annotated comments: one in the target mailbox (stored with
    // mixed case, as parse-email's "Enabled" mode would), one in another mailbox,
    // and one with no "Mailbox ID" property at all (injection was off).
    let comments = [
        mailbox_comment("mb-sales", Some("Sales@Example.com"), false),
        mailbox_comment("mb-support", Some("support@example.com"), false),
        mailbox_comment("mb-none", None, false),
    ]
    .join("\n");
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset.identifier()),
        ],
        comments.as_bytes(),
    );

    // Scope to the sales mailbox, deliberately using a different case to the stored
    // value — the match is case-insensitive (parse-email's Normalized mode lowercases).
    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--mailbox",
        "sales@example.com",
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // Only the sales comment is deleted; the other mailbox and the property-less
    // comment are kept (a comment without the property is never matched).
    let remaining = cli.run(["get", "comments", source.identifier()]);
    assert_eq!(jsonl_count(&remaining), 2, "only the sales comment pruned");
    assert!(!remaining.contains("mb-sales"), "sales comment deleted");
    assert!(remaining.contains("mb-support"), "other mailbox kept");
    assert!(
        remaining.contains("mb-none"),
        "comment without the property kept"
    );

    let dir = run_dir(&backup_parent);
    let comment_backup = only_backup_file(&dir.join("deleted-comments"));
    assert_eq!(
        jsonl_count(&comment_backup),
        1,
        "only the sales comment backed up"
    );
    assert!(comment_backup.contains("mb-sales"));

    // The manifest records the mailbox the run was scoped to, for later audit.
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["mailbox"].as_str(),
        Some("sales@example.com"),
        "manifest records the mailbox scope: {manifest}"
    );

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_mailbox_filters_emails() {
    // Email-side counterpart to the comment test: the mailbox is pushed into the
    // server-side email query, so only the target mailbox's old emails are deleted.
    let cli = TestCli::get();
    let owner = TestCli::project();
    let bucket = format!("{owner}/test-prune-mb-{}", Uuid::new_v4());
    cli.run(["create", "bucket", &bucket]);
    let source = TestSource::new_args(&["--bucket", &bucket]);
    let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    let emails = [
        email_jsonl("e-alice-old", "alice@reinfer.io", "2020-01-01T00:00:00Z"),
        email_jsonl("e-bob-old", "bob@reinfer.io", "2020-01-01T00:00:00Z"),
        email_jsonl("e-alice-recent", "alice@reinfer.io", "2030-01-01T00:00:00Z"),
    ]
    .join("\n");
    cli.run_with_stdin(["create", "emails", "-y", "-b", &bucket], emails.as_bytes());

    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--mailbox",
        "alice@reinfer.io",
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // Only alice's old email is deleted; bob's old email (other mailbox) and alice's
    // recent email (after the cutoff) are kept.
    let remaining = cli.run(["get", "emails", &bucket]);
    assert_eq!(jsonl_count(&remaining), 2, "only alice's old email pruned");
    assert!(
        !remaining.contains("e-alice-old"),
        "alice's old email deleted"
    );
    assert!(remaining.contains("e-bob-old"), "other mailbox kept");
    assert!(remaining.contains("e-alice-recent"), "recent email kept");

    let email_backup = only_backup_file(&run_dir(&backup_parent).join("deleted-emails"));
    assert_eq!(
        jsonl_count(&email_backup),
        1,
        "only alice's old email backed up"
    );
    assert!(email_backup.contains("e-alice-old"));

    // Tear down source + dataset before the bucket (which can't be dropped while a
    // source is attached).
    drop(dataset);
    drop(source);
    let _ = cli.run_and_result(["delete", "bucket", &bucket]);
    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_mailbox_with_include_annotated() {
    // The mailbox filter composes with --include-annotated: it is applied after the
    // annotated-keep check, so an annotated comment in the target mailbox is deleted
    // (because --include-annotated lifts the keep), while an annotated comment in
    // another mailbox is left alone.
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    let comments = [
        mailbox_comment("mb-sales-labelled", Some("sales@example.com"), true),
        mailbox_comment("mb-support-labelled", Some("support@example.com"), true),
    ]
    .join("\n");
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset.identifier()),
        ],
        comments.as_bytes(),
    );

    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--include-annotated",
        "--mailbox",
        "sales@example.com",
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    let remaining = cli.run(["get", "comments", source.identifier()]);
    assert_eq!(jsonl_count(&remaining), 1, "only the sales comment pruned");
    assert!(
        !remaining.contains("mb-sales-labelled"),
        "annotated sales comment deleted"
    );
    assert!(
        remaining.contains("mb-support-labelled"),
        "annotated comment in another mailbox kept"
    );

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_mailbox_dry_run() {
    // --dry-run with --mailbox backs up the mailbox-scoped deletion set without
    // deleting anything.
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    let comments = [
        mailbox_comment("mb-sales", Some("sales@example.com"), false),
        mailbox_comment("mb-support", Some("support@example.com"), false),
    ]
    .join("\n");
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset.identifier()),
        ],
        comments.as_bytes(),
    );

    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--mailbox",
        "sales@example.com",
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--dry-run",
        "--no-progress",
    ]);

    // Nothing deleted, but the backup holds exactly the mailbox-scoped deletion set.
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source.identifier()])),
        2,
        "dry run must not delete comments"
    );
    let comment_backup = only_backup_file(&run_dir(&backup_parent).join("deleted-comments"));
    assert_eq!(
        jsonl_count(&comment_backup),
        1,
        "only the sales comment backed up"
    );
    assert!(comment_backup.contains("mb-sales"));
    assert!(!comment_backup.contains("mb-support"));

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_aborts_when_source_shared_with_out_of_scope_dataset() {
    let cli = TestCli::get();
    let source = TestSource::new();
    let in_scope = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let out_of_scope = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    // The source is also in `out_of_scope` (not passed to --datasets), so prune
    // must refuse rather than touch a dataset the operator didn't list.
    let err = cli.run_and_error([
        "prune",
        "--datasets",
        in_scope.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--dry-run",
        "--no-progress",
    ]);
    assert!(err.contains("Refusing to prune"), "{err}");
    assert!(
        err.contains(out_of_scope.name()),
        "should name the conflicting dataset: {err}"
    );

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_across_batches() {
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    // More old, un-annotated comments than the deletion batch size (32), to
    // exercise batched backup + delete + restore end to end.
    const COUNT: usize = 70;
    let comments = (0..COUNT)
        .map(|i| {
            serde_json::json!({
                "comment": {
                    "id": format!("c{i}"),
                    "timestamp": "2020-01-01T00:00:00Z",
                    "messages": [{"body": {"text": format!("comment {i}")}}],
                }
            })
            .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset.identifier()),
        ],
        comments.as_bytes(),
    );
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source.identifier()])),
        COUNT
    );

    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source.identifier()])),
        0,
        "all old un-annotated comments pruned across batches"
    );

    let comment_backup = only_backup_file(&run_dir(&backup_parent).join("deleted-comments"));
    assert_eq!(
        jsonl_count(&comment_backup),
        COUNT,
        "all pruned comments backed up"
    );

    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--yes",
            "--allow-duplicates",
            &format!("--source={}", source.identifier()),
        ],
        comment_backup.as_bytes(),
    );
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source.identifier()])),
        COUNT,
        "all comments restored from backup"
    );

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_include_annotated_deletes_annotated() {
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset.identifier()),
        ],
        COMMENTS.as_bytes(),
    );
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source.identifier()])),
        3
    );

    // With --include-annotated, both old comments go (annotated and not); only
    // the recent one (after the cutoff) survives.
    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--include-annotated",
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source.identifier()])),
        1,
        "old annotated comment is deleted when --include-annotated is set"
    );

    let dir = run_dir(&backup_parent);
    let comment_backup = only_backup_file(&dir.join("deleted-comments"));
    assert!(comment_backup.contains("old-plain"));
    assert!(
        comment_backup.contains("old-labelled"),
        "annotated comment backed up too"
    );
    // The annotated comment's labels are still captured in the annotation backup,
    // so deleting it remains recoverable.
    let annotation_backup = all_annotations(&dir);
    assert!(
        annotation_backup.contains("old-labelled"),
        "annotations are still backed up under --include-annotated"
    );

    // The manifest records the retention mode, for later audit.
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["include_annotated"].as_bool(),
        Some(true),
        "manifest records the retention mode: {manifest}"
    );

    // Full recovery: restore the deleted comments from the deletion-set backup,
    // then re-apply their annotations from the annotation backup.
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--yes",
            "--allow-duplicates",
            &format!("--source={}", source.identifier()),
        ],
        comment_backup.as_bytes(),
    );
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source.identifier()])),
        3,
        "all comments restored from the deletion-set backup"
    );

    // The comment already exists (restored above), so overwrite to re-attach its
    // labels from the annotation backup.
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--yes",
            "--allow-duplicates",
            "--overwrite",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset.identifier()),
        ],
        annotation_backup.as_bytes(),
    );
    let reviewed = cli.run([
        "get",
        "comments",
        "--reviewed-only",
        "true",
        "--dataset",
        dataset.identifier(),
        source.identifier(),
    ]);
    // `--reviewed-only` returns only annotated comments, so the restored comment
    // appearing here (and nothing else) proves its annotation came back.
    assert_eq!(
        jsonl_count(&reviewed),
        1,
        "exactly one comment is annotated after restore: {reviewed}"
    );
    assert!(
        reviewed.contains("old-labelled"),
        "annotation restored via `re create comments --dataset`: {reviewed}"
    );

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_empty_deletion_set() {
    // A cutoff before all the data: nothing matches, so the deletion set is empty.
    // The run must still succeed — empty backup files are written, verified on the
    // read-back pass, and the manifest records zero counts.
    let cli = TestCli::get();
    let fixture = Fixture::new();
    let backup_parent = temp_backup_dir();

    // 100 years predates every fixture timestamp, so nothing is old enough to prune.
    cli.run([
        "prune",
        "--datasets",
        fixture.dataset().identifier(),
        "--older-than-days",
        "36500",
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // Nothing deleted.
    assert_eq!(fixture.comment_count(), 3, "nothing before the cutoff");
    assert_eq!(fixture.email_count(), 2, "nothing before the cutoff");

    // Empty backup files were written and the manifest counts are zero.
    let dir = run_dir(&backup_parent);
    assert_eq!(
        jsonl_count(&only_backup_file(&dir.join("deleted-comments"))),
        0,
        "empty comment backup"
    );
    assert_eq!(
        jsonl_count(&only_backup_file(&dir.join("deleted-emails"))),
        0,
        "empty email backup"
    );
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["comment_count"].as_u64(), Some(0), "{manifest}");
    assert_eq!(manifest["email_count"].as_u64(), Some(0), "{manifest}");

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_dataset_with_no_sources() {
    // A dataset with no sources has nothing to back up or delete; the run must
    // still complete cleanly (it writes only an empty annotations file).
    let cli = TestCli::get();
    let dataset = TestDataset::new();
    let backup_parent = temp_backup_dir();

    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    let dir = run_dir(&backup_parent);
    // No sources means no annotation files and no comment / email backups.
    assert!(
        annotation_files(&dir).is_empty(),
        "a dataset with no sources produces no annotation files"
    );
    assert_eq!(backup_file_count(&dir.join("deleted-comments")), 0);
    assert_eq!(backup_file_count(&dir.join("deleted-emails")), 0);

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_multiple_datasets_and_buckets() {
    // Two datasets, each with its own source on its own bucket. Exercises the
    // loops over multiple sources / buckets / datasets and the manifest count
    // aggregation — every prior test had exactly one of each.
    let cli = TestCli::get();
    let owner = TestCli::project();
    let backup_parent = temp_backup_dir();

    let bucket_a = format!("{owner}/test-prune-multi-{}", Uuid::new_v4());
    let bucket_b = format!("{owner}/test-prune-multi-{}", Uuid::new_v4());
    cli.run(["create", "bucket", &bucket_a]);
    cli.run(["create", "bucket", &bucket_b]);
    let source_a = TestSource::new_args(&["--bucket", &bucket_a]);
    let source_b = TestSource::new_args(&["--bucket", &bucket_b]);
    let dataset_a = TestDataset::new_args(&[&format!("--source={}", source_a.identifier())]);
    let dataset_b = TestDataset::new_args(&[&format!("--source={}", source_b.identifier())]);

    for (source, dataset) in [(&source_a, &dataset_a), (&source_b, &dataset_b)] {
        cli.run_with_stdin(
            [
                "create",
                "comments",
                "--allow-duplicates",
                "--yes",
                &format!("--source={}", source.identifier()),
                &format!("--dataset={}", dataset.identifier()),
            ],
            COMMENTS.as_bytes(),
        );
    }
    let emails = [
        email_jsonl("e-old", "alice@reinfer.io", "2020-01-01T00:00:00Z"),
        email_jsonl("e-recent", "alice@reinfer.io", "2030-01-01T00:00:00Z"),
    ]
    .join("\n");
    cli.run_with_stdin(["create", "emails", "-y", "-b", &bucket_a], emails.as_bytes());
    cli.run_with_stdin(["create", "emails", "-y", "-b", &bucket_b], emails.as_bytes());

    cli.run([
        "prune",
        "--datasets",
        &format!("{},{}", dataset_a.identifier(), dataset_b.identifier()),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // Both sources and both buckets were pruned.
    for source in [&source_a, &source_b] {
        assert_eq!(
            jsonl_count(&cli.run(["get", "comments", source.identifier()])),
            2,
            "old un-annotated comment pruned from every source"
        );
    }
    for bucket in [&bucket_a, &bucket_b] {
        assert_eq!(
            jsonl_count(&cli.run(["get", "emails", bucket])),
            1,
            "old email pruned from every bucket"
        );
    }

    // A backup file per source, per bucket, and per dataset.
    let dir = run_dir(&backup_parent);
    assert_eq!(backup_file_count(&dir.join("deleted-comments")), 2);
    assert_eq!(backup_file_count(&dir.join("deleted-emails")), 2);
    assert_eq!(
        annotation_files(&dir).len(),
        2,
        "one annotation file per (dataset, source)"
    );
    let comment_backups = all_backup_files(&dir.join("deleted-comments"));
    assert_eq!(jsonl_count(&comment_backups), 2, "one old comment per source");

    // The manifest sums counts across all resources.
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["comment_count"].as_u64(), Some(2), "{manifest}");
    assert_eq!(manifest["email_count"].as_u64(), Some(2), "{manifest}");

    // Tear down datasets + sources before the buckets they depend on.
    drop(dataset_a);
    drop(dataset_b);
    drop(source_a);
    drop(source_b);
    let _ = cli.run_and_result(["delete", "bucket", &bucket_a]);
    let _ = cli.run_and_result(["delete", "bucket", &bucket_b]);
    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_dedups_source_shared_between_in_scope_datasets() {
    // One source belongs to two datasets, both in --datasets (so the shared-source
    // guard does not abort). It must be backed up once (deduped in scope
    // resolution), and a comment annotated in the *second* dataset must still be
    // kept — proving the keep set spans every in-scope dataset, not just the first.
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset_a = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let dataset_b = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    // Both comments land in the shared source; the labelled one is reviewed only in
    // dataset_b. Created against dataset_b so its annotation is recorded there.
    let comments = [
        mailbox_comment("shared-plain", None, false),
        mailbox_comment("shared-labelled", None, true),
    ]
    .join("\n");
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset_b.identifier()),
        ],
        comments.as_bytes(),
    );

    cli.run([
        "prune",
        "--datasets",
        &format!("{},{}", dataset_a.identifier(), dataset_b.identifier()),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // The annotated comment is kept even though it is annotated in the second
    // listed dataset; only the plain comment is pruned.
    let remaining = cli.run(["get", "comments", source.identifier()]);
    assert_eq!(jsonl_count(&remaining), 1, "annotated comment kept: {remaining}");
    assert!(remaining.contains("shared-labelled"), "{remaining}");
    assert!(!remaining.contains("shared-plain"), "{remaining}");

    // The shared source is backed up exactly once despite being in two datasets,
    // while each dataset gets its own annotation file.
    let dir = run_dir(&backup_parent);
    assert_eq!(
        backup_file_count(&dir.join("deleted-comments")),
        1,
        "shared source deduped to one comment backup"
    );
    assert_eq!(
        annotation_files(&dir).len(),
        2,
        "the shared source is backed up once per in-scope dataset"
    );
    let comment_backup = only_backup_file(&dir.join("deleted-comments"));
    assert!(comment_backup.contains("shared-plain"));
    assert!(!comment_backup.contains("shared-labelled"));

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_multi_source_dataset_backs_up_annotations_per_source() {
    // A single dataset spanning two sources. Each source's annotations must land in
    // its own file (`annotations/<dataset>/<source>.jsonl`), never a single mixed
    // file — otherwise a restore via `re create comments --source` would push one
    // source's comments into the other.
    let cli = TestCli::get();
    let source_a = TestSource::new();
    let source_b = TestSource::new();
    let dataset = TestDataset::new_args(&[
        &format!("--source={}", source_a.identifier()),
        &format!("--source={}", source_b.identifier()),
    ]);
    let backup_parent = temp_backup_dir();

    // Each source gets one annotated (kept) and one plain (pruned) old comment.
    for (source, keep, plain) in [
        (&source_a, "a-keep", "a-plain"),
        (&source_b, "b-keep", "b-plain"),
    ] {
        let comments = [
            mailbox_comment(keep, None, true),
            mailbox_comment(plain, None, false),
        ]
        .join("\n");
        cli.run_with_stdin(
            [
                "create",
                "comments",
                "--allow-duplicates",
                "--yes",
                &format!("--source={}", source.identifier()),
                &format!("--dataset={}", dataset.identifier()),
            ],
            comments.as_bytes(),
        );
    }

    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // The plain comment is pruned from each source; the annotated one is kept.
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source_a.identifier()])),
        1,
        "only the plain comment pruned from source A"
    );
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source_b.identifier()])),
        1,
        "only the plain comment pruned from source B"
    );

    // One annotation file per source, and no single file mixes the two sources'
    // comments — each is restorable to exactly one source.
    let dir = run_dir(&backup_parent);
    let files = annotation_files(&dir);
    assert_eq!(files.len(), 2, "one annotation file per source in the dataset");
    for path in &files {
        let contents = fs::read_to_string(path).unwrap();
        assert!(
            !(contents.contains("a-keep") && contents.contains("b-keep")),
            "an annotation file must not mix sources: {path:?}"
        );
    }
    let combined = all_annotations(&dir);
    assert!(combined.contains("a-keep") && combined.contains("b-keep"));

    // Both sources' deletion sets are present, one file each.
    assert_eq!(backup_file_count(&dir.join("deleted-comments")), 2);

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_out_of_scope_source_comments_kept_but_bucket_emails_deleted() {
    // The most surprising destructive behaviour: a source NOT in any listed dataset has
    // its comments kept, but if it shares a bucket with an in-scope source, that
    // bucket's old emails — including ones that fed the out-of-scope source — are still
    // deleted (email deletion is whole-bucket by age). The confirm prompt warns about
    // this; nothing pinned it until now.
    let cli = TestCli::get();
    let owner = TestCli::project();
    let bucket = format!("{owner}/test-prune-scope-{}", Uuid::new_v4());
    cli.run(["create", "bucket", &bucket]);
    // Both sources pull from the SAME bucket; only source_in is in a dataset.
    let source_in = TestSource::new_args(&["--bucket", &bucket]);
    let source_out = TestSource::new_args(&["--bucket", &bucket]);
    let dataset = TestDataset::new_args(&[&format!("--source={}", source_in.identifier())]);
    let backup_parent = temp_backup_dir();

    // One old plain comment in each source, with distinct ids so the assertions are
    // discriminating. source_out is attached to no dataset, so the shared-source guard
    // does not fire.
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source_in.identifier()),
            &format!("--dataset={}", dataset.identifier()),
        ],
        mailbox_comment("in-old", None, false).as_bytes(),
    );
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source_out.identifier()),
        ],
        mailbox_comment("out-old", None, false).as_bytes(),
    );
    let emails = [
        email_jsonl("e-old", "alice@reinfer.io", "2020-01-01T00:00:00Z"),
        email_jsonl("e-recent", "alice@reinfer.io", "2030-01-01T00:00:00Z"),
    ]
    .join("\n");
    cli.run_with_stdin(["create", "emails", "-y", "-b", &bucket], emails.as_bytes());

    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // In-scope source's comment is deleted; the out-of-scope source's comment survives.
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source_in.identifier()])),
        0,
        "in-scope source comment pruned"
    );
    let kept = cli.run(["get", "comments", source_out.identifier()]);
    assert_eq!(jsonl_count(&kept), 1, "out-of-scope source comment kept: {kept}");
    assert!(kept.contains("out-old"));

    // But the shared bucket's old email is deleted regardless of the out-of-scope source.
    assert_eq!(
        jsonl_count(&cli.run(["get", "emails", &bucket])),
        1,
        "old email pruned whole-bucket"
    );

    // Only the in-scope source is backed up, and its file never contains the
    // out-of-scope source's comment.
    let dir = run_dir(&backup_parent);
    assert_eq!(
        backup_file_count(&dir.join("deleted-comments")),
        1,
        "only the in-scope source's comments are backed up"
    );
    let comment_backup = only_backup_file(&dir.join("deleted-comments"));
    assert!(comment_backup.contains("in-old"));
    assert!(!comment_backup.contains("out-old"), "out-of-scope comment never deleted");

    // Tear down sources + dataset before the bucket.
    drop(dataset);
    drop(source_in);
    drop(source_out);
    let _ = cli.run_and_result(["delete", "bucket", &bucket]);
    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_dedups_bucket_shared_between_two_in_scope_sources() {
    // A bucket shared by two in-scope sources (each in its own dataset) must be backed
    // up and deleted from exactly once — only source dedup was previously covered.
    let cli = TestCli::get();
    let owner = TestCli::project();
    let bucket = format!("{owner}/test-prune-shared-bucket-{}", Uuid::new_v4());
    cli.run(["create", "bucket", &bucket]);
    let source_a = TestSource::new_args(&["--bucket", &bucket]);
    let source_b = TestSource::new_args(&["--bucket", &bucket]);
    let dataset_a = TestDataset::new_args(&[&format!("--source={}", source_a.identifier())]);
    let dataset_b = TestDataset::new_args(&[&format!("--source={}", source_b.identifier())]);
    let backup_parent = temp_backup_dir();

    // Emails loaded ONCE into the shared bucket: one old (pruned), one recent (kept).
    let emails = [
        email_jsonl("e-old", "alice@reinfer.io", "2020-01-01T00:00:00Z"),
        email_jsonl("e-recent", "alice@reinfer.io", "2030-01-01T00:00:00Z"),
    ]
    .join("\n");
    cli.run_with_stdin(["create", "emails", "-y", "-b", &bucket], emails.as_bytes());

    cli.run([
        "prune",
        "--datasets",
        &format!("{},{}", dataset_a.identifier(), dataset_b.identifier()),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // The old email is deleted exactly once; the recent one survives.
    assert_eq!(
        jsonl_count(&cli.run(["get", "emails", &bucket])),
        1,
        "old email pruned once from the shared bucket"
    );

    // The shared bucket is deduped: one email backup file, and the manifest counts the
    // old email once — not once per source that references the bucket.
    let dir = run_dir(&backup_parent);
    assert_eq!(
        backup_file_count(&dir.join("deleted-emails")),
        1,
        "shared bucket deduped to one email backup"
    );
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["email_count"].as_u64(), Some(1), "{manifest}");
    assert_eq!(
        manifest["email_backups"].as_array().unwrap().len(),
        1,
        "shared bucket deduped in the manifest: {manifest}"
    );

    drop(dataset_a);
    drop(dataset_b);
    drop(source_a);
    drop(source_b);
    let _ = cli.run_and_result(["delete", "bucket", &bucket]);
    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_mailbox_keeps_annotated_in_target_mailbox() {
    // Under --mailbox WITHOUT --include-annotated, an annotated comment that IS in the
    // target mailbox must still be kept: the annotation-keep check runs before the
    // mailbox match. Only the opposite branch (--include-annotated) was covered.
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    // Both comments are in the sales mailbox; the kept one is annotated (created via
    // --dataset so its uid lands in the keep-set).
    let comments = [
        mailbox_comment("mb-keep", Some("sales@example.com"), true),
        mailbox_comment("mb-plain", Some("sales@example.com"), false),
    ]
    .join("\n");
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset.identifier()),
        ],
        comments.as_bytes(),
    );

    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--mailbox",
        "sales@example.com",
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    let remaining = cli.run(["get", "comments", source.identifier()]);
    assert_eq!(jsonl_count(&remaining), 1, "only the plain comment pruned: {remaining}");
    assert!(
        remaining.contains("mb-keep"),
        "annotated comment in the target mailbox kept: {remaining}"
    );
    assert!(!remaining.contains("mb-plain"), "plain comment in the mailbox pruned");

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_keeps_dismissed_only_reviewed_comment() {
    // "Reviewed" is not the same as "positively labelled": a comment whose only
    // annotation is a dismissed label is still reviewed, so its uid joins the keep-set
    // and it is kept in default mode. Every other annotation fixture uses `assigned`.
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    let comments = [dismissed_comment("dismissed-only"), mailbox_comment("plain", None, false)]
        .join("\n");
    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset.identifier()),
        ],
        comments.as_bytes(),
    );

    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // The dismissed-only comment is reviewed, so it is kept; the plain one is pruned.
    let remaining = cli.run(["get", "comments", source.identifier()]);
    assert_eq!(jsonl_count(&remaining), 1, "only the plain comment pruned: {remaining}");
    assert!(
        remaining.contains("dismissed-only"),
        "dismissed-only reviewed comment kept: {remaining}"
    );

    // ...and it is preserved under annotations/, never the deletion set.
    let dir = run_dir(&backup_parent);
    assert!(all_annotations(&dir).contains("dismissed-only"));
    assert!(!only_backup_file(&dir.join("deleted-comments")).contains("dismissed-only"));

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_bucketless_source_prunes_comments_without_email_deletion() {
    // A source with no bucket (a non-email source) contributes no bucket to scope, so
    // comments are pruned but no email work happens at all.
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    let backup_parent = temp_backup_dir();

    cli.run_with_stdin(
        [
            "create",
            "comments",
            "--allow-duplicates",
            "--yes",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset.identifier()),
        ],
        COMMENTS.as_bytes(),
    );

    cli.run([
        "prune",
        "--datasets",
        dataset.identifier(),
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);

    // The old un-annotated comment is pruned and backed up.
    assert_eq!(
        jsonl_count(&cli.run(["get", "comments", source.identifier()])),
        2,
        "old un-annotated comment pruned"
    );
    let dir = run_dir(&backup_parent);
    assert!(only_backup_file(&dir.join("deleted-comments")).contains("old-plain"));

    // No bucket in scope => no email backup files and a zero email count.
    assert_eq!(
        backup_file_count(&dir.join("deleted-emails")),
        0,
        "a bucket-less source does no email work"
    );
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["email_count"].as_u64(), Some(0), "{manifest}");

    fs::remove_dir_all(&backup_parent).ok();
}

#[test]
fn test_prune_unknown_dataset_aborts_cleanly() {
    // A bad --datasets identifier must abort during scope resolution, before any backup
    // directory is created — so a typo never leaves a half-made backup tree.
    let cli = TestCli::get();
    let owner = TestCli::project();
    let backup_parent = temp_backup_dir();
    let missing = format!("{owner}/does-not-exist-{}", Uuid::new_v4());

    let err = cli.run_and_error([
        "prune",
        "--datasets",
        &missing,
        "--older-than-days",
        OLDER_THAN_DAYS,
        "--backup-dir",
        backup_parent.to_str().unwrap(),
        "--yes",
        "--no-progress",
    ]);
    assert!(err.contains("Could not get dataset"), "{err}");
    assert!(
        !backup_parent.exists(),
        "no backup directory is created when scope resolution fails"
    );

    fs::remove_dir_all(&backup_parent).ok();
}
