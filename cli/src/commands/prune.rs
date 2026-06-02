use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use colored::Colorize;
use dialoguer::Confirm;
use log::info;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

use reinfer_client::{
    AnnotatedComment, Bucket, BucketId, BucketIdentifier, Client, Comment, CommentId, CommentUid,
    CommentsIter, CommentsIterTimerange, Dataset, DatasetId, DatasetIdentifier, EmailId,
    EmailsQueryFilter, PropertyValue, Source, SourceId, SourceIdentifier,
};

use crate::{
    commands::get::comments::download_reviewed_comments,
    progress::{Options as ProgressOptions, Progress},
};

// The maximum number of ids the API permits deleting in a single call.
const DELETION_BATCH_SIZE: usize = 32;

// The comment user property that parse-email writes the originating mailbox into
// (when mailbox-id injection is enabled). Mirrors `USER_PROPERTY_MAILBOX_NAME`
// in the platform's parse-email service. The CLI's `PropertyMap` strips the
// `string:` type prefix, so the in-memory key is the bare name.
const MAILBOX_USER_PROPERTY: &str = "Mailbox ID";

#[derive(Debug, StructOpt)]
pub struct PruneArgs {
    #[structopt(long = "datasets", use_delimiter = true)]
    /// Datasets in scope (comma-separated names or ids). Every source and bucket
    /// referenced by these datasets is pruned.
    datasets: Vec<DatasetIdentifier>,

    #[structopt(long = "older-than-days")]
    /// Delete data older than this many days (cutoff = now - N days).
    older_than_days: u32,

    #[structopt(long = "include-annotated")]
    /// Also delete comments that are annotated. By default annotated comments are
    /// kept; pass this for true age-based retention. "Annotated" is scoped to the
    /// datasets you can see (which, after the shared-source check, are the
    /// --datasets): a comment is kept only if it is reviewed in one of those. A
    /// comment annotated only in a dataset you can't access is treated as
    /// un-annotated and deleted in the default mode.
    include_annotated: bool,

    #[structopt(long = "mailbox")]
    /// Restrict the run to a single mailbox. Emails are filtered server-side by
    /// mailbox name; comments are matched against their "Mailbox ID" user property
    /// (written by email parsing when mailbox-id injection is enabled), so a comment
    /// without that property is treated as not in the mailbox and is kept. Matching
    /// is case-insensitive.
    mailbox: Option<String>,

    #[structopt(long = "backup-dir", parse(from_os_str))]
    /// Directory under which a timestamped, write-once backup folder is created.
    backup_dir: PathBuf,

    #[structopt(long = "dry-run")]
    /// Back up and report what would be deleted, without deleting anything.
    dry_run: bool,

    #[structopt(short = "y", long = "yes")]
    /// Skip the confirmation prompt. Required for non-interactive use.
    yes: bool,

    #[structopt(long = "no-progress")]
    /// Don't display progress bars.
    no_progress: bool,
}

/// The resolved set of resources a prune run operates on.
struct Scope {
    datasets: Vec<Dataset>,
    sources: Vec<Source>,
    buckets: Vec<Bucket>,
}

/// Record of the backup written for one source or bucket. The delete step reads
/// the file back to source the ids it deletes, so it can only ever delete data
/// that is physically present in the backup.
#[derive(Serialize)]
struct BackupFile {
    /// Full name of the source (comments) or bucket (emails) this covers.
    resource: String,
    /// Path to the backup file, relative to the backup directory.
    file: String,
    count: usize,
    crc32: u32,
}

/// Summary of a prune run. Holds per-file counts and checksums — not the ids
/// themselves, so it stays small regardless of how much data is pruned.
#[derive(Serialize)]
struct Manifest {
    run_id: String,
    cutoff: DateTime<Utc>,
    include_annotated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    mailbox: Option<String>,
    datasets: Vec<String>,
    annotation_backups: Vec<BackupFile>,
    comment_backups: Vec<BackupFile>,
    email_backups: Vec<BackupFile>,
    comment_count: usize,
    email_count: usize,
}

pub fn run(args: &PruneArgs, client: Client) -> Result<()> {
    validate_args(args)?;
    let cutoff = resolve_cutoff(args)?;
    let run_id = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let show_progress = !args.no_progress;

    let scope = resolve_scope(&client, &args.datasets)?;
    validate_shared_sources(&client, &scope)?;

    // Confirm up front (before the potentially slow backup), unless it's a dry
    // run (deletes nothing) or the user passed --yes.
    if !args.dry_run && !args.yes {
        confirm_prune(
            &scope,
            cutoff,
            args.include_annotated,
            args.mailbox.as_deref(),
        )?;
    }

    let backup_dir = create_backup_dir(&args.backup_dir, &run_id)?;
    info!("Backing up to `{}`", backup_dir.display());

    let (annotation_backups, reviewed_uids) =
        back_up_annotations(&client, &scope, &backup_dir, show_progress)?;
    let manifest = back_up_deletion_set(
        &client,
        &scope,
        cutoff,
        args.include_annotated,
        args.mailbox.as_deref(),
        &run_id,
        &backup_dir,
        annotation_backups,
        &reviewed_uids,
        show_progress,
    )?;
    write_manifest(&backup_dir, &manifest)?;

    // Surface the mailbox scope in the run-completion logs too, so the operator's
    // console (not just the manifest) records which mailbox was pruned.
    let mailbox_note = match args.mailbox.as_deref() {
        Some(mailbox) => format!(" in mailbox `{mailbox}`"),
        None => String::new(),
    };

    if args.dry_run {
        info!(
            "Dry run complete: {} comments and {} emails{} would be deleted (backup at `{}`).",
            manifest.comment_count,
            manifest.email_count,
            mailbox_note,
            backup_dir.display()
        );
        return Ok(());
    }

    delete_from_backups(&client, &scope, &backup_dir, &manifest, show_progress)?;
    info!(
        "Pruned {} comments and {} emails{}.",
        manifest.comment_count, manifest.email_count, mailbox_note
    );
    Ok(())
}

/// Validate arguments that structopt can't enforce on their own. Split out so the
/// guard can be unit-tested without constructing a client.
fn validate_args(args: &PruneArgs) -> Result<()> {
    if args.datasets.is_empty() {
        bail!("At least one dataset must be given via --datasets.");
    }
    Ok(())
}

fn resolve_cutoff(args: &PruneArgs) -> Result<DateTime<Utc>> {
    // `checked_sub_signed` rather than `-`: chrono's `Sub` for `DateTime` panics on
    // overflow, so a very large `--older-than-days` would abort with a backtrace
    // instead of a clean error.
    Utc::now()
        .checked_sub_signed(Duration::days(i64::from(args.older_than_days)))
        .context("`--older-than-days` is too large: the cutoff date is out of range")
}

/// Resolve the in-scope datasets to their sources and (for email sources) buckets.
fn resolve_scope(client: &Client, dataset_identifiers: &[DatasetIdentifier]) -> Result<Scope> {
    let datasets = dataset_identifiers
        .iter()
        .map(|identifier| {
            client
                .get_dataset(identifier.clone())
                .with_context(|| format!("Could not get dataset `{identifier}`"))
        })
        .collect::<Result<Vec<_>>>()?;

    // Unique source ids across all in-scope datasets.
    let source_ids: HashSet<_> = datasets
        .iter()
        .flat_map(|dataset| dataset.source_ids.iter().cloned())
        .collect();

    let sources = source_ids
        .into_iter()
        .map(|source_id| {
            client
                .get_source(SourceIdentifier::Id(source_id))
                .context("Could not get source")
        })
        .collect::<Result<Vec<_>>>()?;

    // Unique buckets referenced by those sources.
    let bucket_ids: HashSet<_> = sources
        .iter()
        .filter_map(|source| source.bucket_id.clone())
        .collect();

    let buckets = bucket_ids
        .into_iter()
        .map(|bucket_id| {
            client
                .get_bucket(BucketIdentifier::Id(bucket_id))
                .context("Could not get bucket")
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Scope {
        datasets,
        sources,
        buckets,
    })
}

/// Best-effort check that no in-scope source is shared with a dataset that is
/// not in scope. Aborts on a *visible* conflict. (Datasets in projects the
/// operator cannot read are invisible to this check; the prune confirmation
/// spells that caveat out.)
fn validate_shared_sources(client: &Client, scope: &Scope) -> Result<()> {
    let in_scope_dataset_ids: HashSet<_> = scope.datasets.iter().map(|d| d.id.clone()).collect();
    let in_scope_source_ids: HashSet<_> = scope.sources.iter().map(|s| s.id.clone()).collect();

    let all_datasets = client
        .get_datasets()
        .context("Could not list datasets to check for shared sources")?;

    let mut conflicts: Vec<String> = Vec::new();
    for dataset in &all_datasets {
        if in_scope_dataset_ids.contains(&dataset.id) {
            continue;
        }
        if dataset
            .source_ids
            .iter()
            .any(|source_id| in_scope_source_ids.contains(source_id))
        {
            conflicts.push(dataset.full_name().0);
        }
    }

    if !conflicts.is_empty() {
        bail!(
            "Refusing to prune: in-scope sources are also used by datasets not in --datasets: {}. \
             Add them to --datasets or narrow the scope.",
            conflicts.join(", ")
        );
    }
    Ok(())
}

/// Summarise what the run will delete and the non-obvious caveats, and ask the
/// operator to confirm. Shown before the backup so they aren't surprised after
/// walking away; suppressed by `--yes`.
fn confirm_prune(
    scope: &Scope,
    cutoff: DateTime<Utc>,
    include_annotated: bool,
    mailbox: Option<&str>,
) -> Result<()> {
    let datasets = scope
        .datasets
        .iter()
        .map(|dataset| dataset.full_name().0)
        .collect::<Vec<_>>();
    let prompt = confirm_prompt(
        &datasets,
        scope.sources.len(),
        scope.buckets.len(),
        cutoff,
        include_annotated,
        mailbox,
    );
    decide(Confirm::new().with_prompt(prompt).interact()?)
}

/// Map the operator's yes/no answer to a result. Split from `confirm_prune` so the
/// abort path is unit-testable without a terminal.
fn decide(confirmed: bool) -> Result<()> {
    if confirmed {
        Ok(())
    } else {
        bail!("Prune aborted by user.");
    }
}

/// Build the confirmation message. Split out from `confirm_prune` (which adds the
/// interactive prompt) so the branching — mailbox vs whole-bucket, the annotated
/// caveat — can be unit-tested without a terminal.
fn confirm_prompt(
    datasets: &[String],
    sources: usize,
    buckets: usize,
    cutoff: DateTime<Utc>,
    include_annotated: bool,
    mailbox: Option<&str>,
) -> String {
    let datasets_count = datasets.len();
    let datasets = datasets.join(", ");
    let cutoff = cutoff.format("%Y-%m-%dT%H:%M:%SZ");
    let annotated_note = if include_annotated {
        "\n\nAnnotated comments will be deleted too (--include-annotated), including ones annotated \
         only in datasets you can't access — whose annotations are NOT backed up."
    } else {
        ""
    };
    // The mailbox filter changes who is in scope, so the intro line, the
    // email-deletion bullet, and an extra comment-matching caveat all depend on it.
    let (mailbox_clause, email_bullet, mailbox_bullet) = match mailbox {
        Some(mailbox) => (
            format!(" in mailbox `{mailbox}`"),
            format!(
                "Email deletion is restricted to mailbox `{mailbox}`: only emails in that mailbox \
                 dated before the cutoff are deleted."
            ),
            format!(
                "\n  - Comments are matched to the mailbox via their `Mailbox ID` user property \
                 (written by email\n    parsing when mailbox-id injection is enabled). A comment \
                 without it is treated as not in\n    mailbox `{mailbox}` and is kept — so if \
                 injection was off for these sources, NO comments are deleted."
            ),
        ),
        None => (
            String::new(),
            "Email deletion is whole-bucket by age: every email before the cutoff is deleted from \
             each\n    bucket, including emails feeding other sources or datasets — in scope or not."
                .to_owned(),
            String::new(),
        ),
    };
    format!(
        "About to back up and then delete comments and emails dated before {cutoff}{mailbox_clause} \
from {sources} source(s) and {buckets} bucket(s) across {datasets_count} dataset(s): \
{datasets}.{annotated_note}

WARNING:
  - Comment attachment content is not backed up — only attachment metadata.
  - A comment is kept only if it's annotated in a dataset you listed. A comment annotated only
    in a dataset you can't access is treated as un-annotated: it is deleted, and that annotation
    is NOT backed up.
  - {email_bullet}{mailbox_bullet}
  - Only datasets you can read are checked for shared sources. If an in-scope source also belongs
    to a dataset in a project you can't access, that dataset loses the comments deleted here.
  - Sources not in any listed dataset are out of scope — their comments are NOT deleted, even if
    their bucket's emails are.

The backup is your only way to undo this, and it contains personal data. Write it somewhere secure \
and keep it safe — restore is manual, via `re create comments` / `re create emails`.

The backup is written before anything is deleted. Proceed?"
    )
}

/// Create a fresh, run-stamped backup directory. Refuses to reuse an existing
/// one so a run never overwrites a previous backup.
fn create_backup_dir(root: &Path, run_id: &str) -> Result<PathBuf> {
    let dir = root.join(run_id);
    if dir.exists() {
        bail!("Backup directory `{}` already exists", dir.display());
    }
    fs::create_dir_all(&dir)
        .with_context(|| format!("Could not create backup directory `{}`", dir.display()))?;
    Ok(dir)
}

/// Relative path of a per-source comment deletion-set backup.
fn comment_backup_file(source_id: &SourceId) -> String {
    format!("deleted-comments/{}.jsonl", source_id.0)
}

/// Relative path of a per-bucket email deletion-set backup.
fn email_backup_file(bucket_id: &BucketId) -> String {
    format!("deleted-emails/{}.jsonl", bucket_id.0)
}

/// Relative path of an annotation backup — one file per `(dataset, source)` pair,
/// keyed by source id under a per-dataset directory. This mirrors the per-source
/// layout of the comment deletion-set backups (`deleted-comments/<source>.jsonl`),
/// so every file holds exactly one source's records and restores with a single
/// `re create comments --source <source> --dataset <dataset>`. A source shared by
/// several in-scope datasets is backed up once per dataset, since a comment can be
/// reviewed differently in each.
fn annotation_backup_file(dataset_id: &DatasetId, source_id: &SourceId) -> String {
    format!("annotations/{}/{}.jsonl", dataset_id.0, source_id.0)
}

/// Back up every reviewed (annotated) comment for each in-scope dataset,
/// recording each file's count and checksum so it can be verified before delete.
///
/// Also returns the set of every backed-up comment's `uid`. Because the
/// shared-source check has already guaranteed that every dataset the operator can
/// see which references an in-scope source is one of `--datasets`, this set is
/// exactly "the comments annotated in a dataset the operator can see". The
/// deletion pass keeps those (unless `--include-annotated`) rather than relying on
/// the comment's global `has_annotations` flag, which would also count annotations
/// in datasets the operator can't access. The set is bounded by the number of
/// reviewed comments (human-labelled, so far smaller than the total comment count).
fn back_up_annotations(
    client: &Client,
    scope: &Scope,
    backup_dir: &Path,
    show_progress: bool,
) -> Result<(Vec<BackupFile>, HashSet<CommentUid>)> {
    fs::create_dir_all(backup_dir.join("annotations"))
        .context("Could not create annotations backup directory")?;

    let mut reviewed_uids = HashSet::new();
    let mut annotation_backups = Vec::new();
    for dataset in &scope.datasets {
        // One file per source in the dataset, matching the per-source granularity
        // of the deletion-set backups so each file restores to a single source.
        // We filter `scope.sources` (rather than iterate `dataset.source_ids`)
        // because the body needs the full `Source` for `full_name()`, not just
        // the id; `scope.sources` is the union of all in-scope datasets' sources,
        // so this selects exactly the dataset's sources.
        for source in scope
            .sources
            .iter()
            .filter(|source| dataset.source_ids.contains(&source.id))
        {
            let relative_path = annotation_backup_file(&dataset.id, &source.id);
            let absolute_path = backup_dir.join(&relative_path);
            fs::create_dir_all(
                absolute_path
                    .parent()
                    .expect("annotation path has a parent"),
            )
            .context("Could not create annotations backup directory")?;
            {
                let mut writer = create_backup_writer(&absolute_path)?;
                // The dataset-query endpoint is addressed by `owner/name`, so pass
                // full names (not ids) for both the dataset and its source.
                download_reviewed_comments(
                    client,
                    SourceIdentifier::FullName(source.full_name()),
                    DatasetIdentifier::FullName(dataset.full_name()),
                    show_progress,
                    &mut writer,
                )
                .with_context(|| {
                    format!(
                        "Could not back up annotations for source `{}` in dataset `{}`",
                        source.full_name().0,
                        dataset.full_name().0
                    )
                })?;
                writer
                    .flush()
                    .context("Could not flush annotations backup")?;
            }
            let (count, crc32) = crc_count_and_uids(&absolute_path, &mut reviewed_uids)?;
            annotation_backups.push(BackupFile {
                resource: source.full_name().0,
                file: relative_path,
                count,
                crc32,
            });
        }
    }
    Ok((annotation_backups, reviewed_uids))
}

/// Select the data to delete (comments and emails older than the cutoff —
/// excluding annotated comments unless `include_annotated`) and stream it to
/// backup files, recording per-file counts and checksums in the manifest. Ids
/// are never held in memory.
#[allow(clippy::too_many_arguments)]
fn back_up_deletion_set(
    client: &Client,
    scope: &Scope,
    cutoff: DateTime<Utc>,
    include_annotated: bool,
    mailbox: Option<&str>,
    run_id: &str,
    backup_dir: &Path,
    annotation_backups: Vec<BackupFile>,
    reviewed_uids: &HashSet<CommentUid>,
    show_progress: bool,
) -> Result<Manifest> {
    fs::create_dir_all(backup_dir.join("deleted-comments"))
        .context("Could not create deleted-comments directory")?;
    fs::create_dir_all(backup_dir.join("deleted-emails"))
        .context("Could not create deleted-emails directory")?;

    // The page iterators don't expose a total upfront, so the bar reports a
    // running count of records backed up rather than progress towards a target.
    let backed_up = Arc::new(AtomicUsize::new(0));
    let _progress = show_progress.then(|| count_progress_bar(&backed_up, "backed up"));

    let mut comment_backups = Vec::new();
    let mut comment_count = 0;
    for source in &scope.sources {
        let rel = comment_backup_file(&source.id);
        let mut backup = BackupWriter::create(&backup_dir.join(&rel))?;
        let timerange = CommentsIterTimerange {
            from: None,
            to: Some(cutoff),
        };
        for page in client.get_comments_iter(
            &source.full_name(),
            Some(CommentsIter::MAX_PAGE_SIZE),
            timerange,
        ) {
            for comment in page.context("Could not page comments")? {
                // Keep comments reviewed in a dataset the operator can see (their
                // uid is in the annotation backup). Unlike the global
                // `comment.has_annotations`, this does not protect comments
                // annotated only in datasets the operator can't access.
                if !include_annotated && reviewed_uids.contains(&comment.uid) {
                    continue;
                }
                // When scoped to a mailbox, only delete comments positively
                // attributed to it via their "Mailbox ID" user property. A comment
                // without that property is never matched, so it is kept.
                // Filtering client side due to no existing mailbox filter when querying source
                // comments
                if let Some(mailbox) = mailbox {
                    if !comment_in_mailbox(&comment, mailbox) {
                        continue;
                    }
                }
                // Write in the same shape `re get comments` emits so the backup
                // restores via `re create comments`.
                backup.write_record(&AnnotatedComment {
                    comment,
                    labelling: None,
                    entities: None,
                    thread_properties: None,
                    moon_forms: None,
                    label_properties: None,
                })?;
                backed_up.fetch_add(1, Ordering::SeqCst);
            }
        }
        let (count, crc32) = backup.finish()?;
        comment_count += count;
        comment_backups.push(BackupFile {
            resource: source.full_name().0,
            file: rel,
            count,
            crc32,
        });
    }

    let mut email_backups = Vec::new();
    let mut email_count = 0;
    for bucket in &scope.buckets {
        let rel = email_backup_file(&bucket.id);
        let mut backup = BackupWriter::create(&backup_dir.join(&rel))?;
        let filter = EmailsQueryFilter {
            from_timestamp: None,
            to_timestamp: Some(cutoff),
            mailbox_name: mailbox.map(str::to_owned),
        };
        for page in client.query_emails_iter(&bucket.full_name(), filter, None) {
            for email in page.context("Could not page emails")? {
                // Write the full email (including mime_content) in the shape
                // `re get emails` emits, so the backup restores via `re create emails`.
                backup.write_record(&email)?;
                backed_up.fetch_add(1, Ordering::SeqCst);
            }
        }
        let (count, crc32) = backup.finish()?;
        email_count += count;
        email_backups.push(BackupFile {
            resource: bucket.full_name().0,
            file: rel,
            count,
            crc32,
        });
    }

    Ok(Manifest {
        run_id: run_id.to_owned(),
        cutoff,
        include_annotated,
        mailbox: mailbox.map(str::to_owned),
        datasets: scope.datasets.iter().map(|d| d.full_name().0).collect(),
        annotation_backups,
        comment_backups,
        email_backups,
        comment_count,
        email_count,
    })
}

/// Whether a comment belongs to `mailbox`, matched against its "Mailbox ID" user
/// property. The match is case-insensitive because parse-email's "Normalized" mode
/// lowercases the stored value; ASCII case-folding suffices, as this property holds
/// email-address mailbox names. A comment with no such property (e.g. mailbox-id
/// injection was disabled) never matches, so a mailbox-scoped run only ever deletes
/// comments it can positively attribute to the mailbox.
fn comment_in_mailbox(comment: &Comment, mailbox: &str) -> bool {
    matches!(
        comment.user_properties.get(MAILBOX_USER_PROPERTY),
        Some(PropertyValue::String(value)) if value.eq_ignore_ascii_case(mailbox)
    )
}

/// Streams JSONL records to a backup file, tracking a CRC32 and a count over the
/// exact bytes written so the delete step can verify them on read-back.
struct BackupWriter {
    writer: BufWriter<File>,
    hasher: crc32fast::Hasher,
    count: usize,
}

impl BackupWriter {
    fn create(path: &Path) -> Result<Self> {
        Ok(Self {
            writer: create_backup_writer(path)?,
            hasher: crc32fast::Hasher::new(),
            count: 0,
        })
    }

    fn write_record<T: Serialize>(&mut self, record: &T) -> Result<()> {
        let mut line = serde_json::to_vec(record).context("Could not serialise backup record")?;
        line.push(b'\n');
        self.hasher.update(&line);
        self.writer
            .write_all(&line)
            .context("Could not write backup record")?;
        self.count += 1;
        Ok(())
    }

    fn finish(mut self) -> Result<(usize, u32)> {
        self.writer.flush().context("Could not flush backup file")?;
        Ok((self.count, self.hasher.finalize()))
    }
}

/// A running-count progress bar (no upfront total) for a phase that streams an
/// unknown number of records — `verb` is the past-tense action shown, e.g.
/// "backed up" or "deleted".
fn count_progress_bar(counter: &Arc<AtomicUsize>, verb: &'static str) -> Progress {
    Progress::new(
        move |counter: &AtomicUsize| {
            let count = counter.load(Ordering::SeqCst) as u64;
            (
                count,
                format!("{} {}", count.to_string().bold(), verb.dimmed()),
            )
        },
        counter,
        None,
        ProgressOptions { bytes_units: false },
    )
}

fn write_manifest(backup_dir: &Path, manifest: &Manifest) -> Result<()> {
    let path = backup_dir.join("manifest.json");
    let mut writer = create_backup_writer(&path)?;
    serde_json::to_writer_pretty(&mut writer, manifest).context("Could not write manifest")?;
    writer.flush().context("Could not flush manifest")?;
    Ok(())
}

/// Verify every backup file's integrity, then delete from them. Verification is
/// a complete read-only pass over all files first, so nothing is deleted until
/// the entire backup is confirmed intact. The delete pass then re-streams each
/// file and sources the ids it deletes from the backup, so it can only ever
/// delete data that was successfully backed up.
fn delete_from_backups(
    client: &Client,
    scope: &Scope,
    backup_dir: &Path,
    manifest: &Manifest,
    show_progress: bool,
) -> Result<()> {
    verify_then_delete(
        backup_dir,
        manifest,
        show_progress,
        |resource, ids| {
            let source = scope
                .sources
                .iter()
                .find(|source| source.full_name().0 == resource)
                .context("Internal error: backup resource is not in scope")?;
            client
                .delete_comments(source, ids)
                .context("Could not delete comments")
        },
        |resource, ids| {
            let bucket = scope
                .buckets
                .iter()
                .find(|bucket| bucket.full_name().0 == resource)
                .context("Internal error: backup resource is not in scope")?;
            client
                .delete_emails(bucket.full_name(), ids)
                .context("Could not delete emails")
        },
    )
}

/// Verify all backups (PASS 1), then — only if every file is intact — delete from
/// them (PASS 2). Separating the passes by resource kind is the whole safety
/// guarantee: a single corrupt comment, email, or annotation backup aborts the run
/// before any resource is touched. The `delete_*` callbacks receive a backup's
/// resource full-name and a batch of ids; injecting them (instead of a `Client`)
/// lets the composed verify-then-delete invariant be tested without a backend.
fn verify_then_delete<DeleteComments, DeleteEmails>(
    backup_dir: &Path,
    manifest: &Manifest,
    show_progress: bool,
    delete_comments: DeleteComments,
    delete_emails: DeleteEmails,
) -> Result<()>
where
    DeleteComments: FnMut(&str, &[CommentId]) -> Result<()>,
    DeleteEmails: FnMut(&str, &[EmailId]) -> Result<()>,
{
    verify_all_backups(backup_dir, manifest)?;
    delete_verified_backups(
        backup_dir,
        manifest,
        show_progress,
        delete_comments,
        delete_emails,
    )
}

/// PASS 1: verify every annotation, comment, and email backup against the manifest
/// without deleting anything. Returns on the first failure, so any single corrupt
/// file fails the whole pass before PASS 2 runs. Annotation backups are verified for
/// integrity too, though they are never deleted from.
fn verify_all_backups(backup_dir: &Path, manifest: &Manifest) -> Result<()> {
    for backup in &manifest.annotation_backups {
        verify_backup(&backup_dir.join(&backup.file), backup)?;
    }
    for backup in &manifest.comment_backups {
        for_each_verified_batch(&backup_dir.join(&backup.file), backup, comment_id, |_| {
            Ok(())
        })?;
    }
    for backup in &manifest.email_backups {
        for_each_verified_batch(&backup_dir.join(&backup.file), backup, email_id, |_| Ok(()))?;
    }
    Ok(())
}

/// PASS 2: re-stream each verified backup and delete the ids it contains, sourcing
/// every id from the backup file (never an in-memory list). Must run only after
/// `verify_all_backups` has confirmed every file.
fn delete_verified_backups<DeleteComments, DeleteEmails>(
    backup_dir: &Path,
    manifest: &Manifest,
    show_progress: bool,
    mut delete_comments: DeleteComments,
    mut delete_emails: DeleteEmails,
) -> Result<()>
where
    DeleteComments: FnMut(&str, &[CommentId]) -> Result<()>,
    DeleteEmails: FnMut(&str, &[EmailId]) -> Result<()>,
{
    let deleted = Arc::new(AtomicUsize::new(0));
    let _progress = show_progress.then(|| count_progress_bar(&deleted, "deleted"));
    for backup in &manifest.comment_backups {
        for_each_verified_batch(&backup_dir.join(&backup.file), backup, comment_id, |ids| {
            delete_comments(&backup.resource, ids)?;
            deleted.fetch_add(ids.len(), Ordering::SeqCst);
            Ok(())
        })?;
    }
    for backup in &manifest.email_backups {
        for_each_verified_batch(&backup_dir.join(&backup.file), backup, email_id, |ids| {
            delete_emails(&backup.resource, ids)?;
            deleted.fetch_add(ids.len(), Ordering::SeqCst);
            Ok(())
        })?;
    }
    Ok(())
}

/// One id read back from a backup line. Email backups (full `Email`) carry the id
/// at the top level; comment backups (`AnnotatedComment`) nest it under `comment`.
#[derive(Deserialize)]
struct IdField<Id> {
    id: Id,
}

#[derive(Deserialize)]
struct CommentLine {
    comment: IdField<CommentId>,
}

/// An annotation backup line — we only need the comment's globally-unique `uid` to
/// decide which comments the deletion pass keeps.
#[derive(Deserialize)]
struct UidLine {
    comment: UidField,
}

#[derive(Deserialize)]
struct UidField {
    uid: CommentUid,
}

fn comment_id(line: CommentLine) -> CommentId {
    line.comment.id
}

fn email_id(line: IdField<EmailId>) -> EmailId {
    line.id
}

/// Stream a backup file in batches of ids — `extract_id` pulls the id out of each
/// parsed line — invoke `on_batch` for each batch, and verify the file's CRC32 and
/// count against the manifest. Called first with a no-op `on_batch` to verify, then
/// with the deleter to act on the verified ids.
fn for_each_verified_batch<Line, Id, OnBatch>(
    path: &Path,
    expected: &BackupFile,
    extract_id: impl Fn(Line) -> Id,
    mut on_batch: OnBatch,
) -> Result<()>
where
    Line: DeserializeOwned,
    OnBatch: FnMut(&[Id]) -> Result<()>,
{
    let mut batch: Vec<Id> = Vec::with_capacity(DELETION_BATCH_SIZE);
    let (count, crc32) = stream_backup_lines(path, |line| {
        let parsed: Line = serde_json::from_slice(line)
            .with_context(|| format!("Corrupt backup line in `{}`", path.display()))?;
        batch.push(extract_id(parsed));
        if batch.len() >= DELETION_BATCH_SIZE {
            on_batch(&batch)?;
            batch.clear();
        }
        Ok(())
    })?;
    if !batch.is_empty() {
        on_batch(&batch)?;
    }

    check_integrity(path, expected, count, crc32)
}

/// Stream a backup file line by line, updating a CRC32 over the raw bytes (newlines
/// included — the same bytes a `BackupWriter` hashes while writing) and invoking
/// `per_line` for each line. Returns the line count and final CRC32.
fn stream_backup_lines(
    path: &Path,
    mut per_line: impl FnMut(&[u8]) -> Result<()>,
) -> Result<(usize, u32)> {
    let mut reader = BufReader::new(
        File::open(path).with_context(|| format!("Could not open backup `{}`", path.display()))?,
    );
    let mut hasher = crc32fast::Hasher::new();
    let mut count = 0;
    let mut line = Vec::new();
    loop {
        line.clear();
        if reader
            .read_until(b'\n', &mut line)
            .with_context(|| format!("Could not read backup `{}`", path.display()))?
            == 0
        {
            break;
        }
        hasher.update(&line);
        per_line(&line)?;
        count += 1;
    }
    Ok((count, hasher.finalize()))
}

/// Bail unless a backup file's observed count and CRC32 match its manifest entry.
fn check_integrity(path: &Path, expected: &BackupFile, count: usize, crc32: u32) -> Result<()> {
    if count != expected.count || crc32 != expected.crc32 {
        bail!(
            "Backup `{}` failed verification (count/checksum mismatch with the manifest)",
            path.display()
        );
    }
    Ok(())
}

fn create_backup_writer(path: &Path) -> Result<BufWriter<File>> {
    File::create(path)
        .with_context(|| format!("Could not create backup file `{}`", path.display()))
        .map(BufWriter::new)
}

/// Stream a file, returning its line count and CRC32 over the raw bytes (newlines
/// included) — the same bytes a `BackupWriter` hashes while writing.
fn crc_and_count(path: &Path) -> Result<(usize, u32)> {
    stream_backup_lines(path, |_| Ok(()))
}

/// Like `crc_and_count`, but also parses each line and collects its comment `uid`
/// into `reviewed`. Used while checksumming the annotation backups so the deletion
/// pass can keep comments reviewed in a dataset the operator can see.
fn crc_count_and_uids(path: &Path, reviewed: &mut HashSet<CommentUid>) -> Result<(usize, u32)> {
    stream_backup_lines(path, |line| {
        let parsed: UidLine = serde_json::from_slice(line)
            .with_context(|| format!("Corrupt annotation backup line in `{}`", path.display()))?;
        reviewed.insert(parsed.comment.uid);
        Ok(())
    })
}

/// Verify a backup file's count and CRC32 against the manifest, without parsing
/// its contents. Used for annotation backups (which are not deleted, so their ids
/// are never read).
fn verify_backup(path: &Path, expected: &BackupFile) -> Result<()> {
    let (count, crc32) = crc_and_count(path)?;
    check_integrity(path, expected, count, crc32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn temp_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("re-prune-unit-{tag}-{}", uuid::Uuid::new_v4()))
    }

    fn args(older_than_days: u32) -> PruneArgs {
        PruneArgs {
            datasets: vec![],
            older_than_days,
            include_annotated: false,
            mailbox: None,
            backup_dir: PathBuf::new(),
            dry_run: false,
            yes: false,
            no_progress: false,
        }
    }

    /// Write `{ "id": ... }` lines via `BackupWriter`, returning (count, crc32).
    fn write_id_backup(path: &Path, ids: &[&str]) -> (usize, u32) {
        let mut writer = BackupWriter::create(path).unwrap();
        for id in ids {
            writer
                .write_record(&serde_json::json!({ "id": id }))
                .unwrap();
        }
        writer.finish().unwrap()
    }

    fn backup_file(count: usize, crc32: u32) -> BackupFile {
        BackupFile {
            resource: "owner/thing".to_owned(),
            file: "f.jsonl".to_owned(),
            count,
            crc32,
        }
    }

    #[test]
    fn cutoff_older_than_days_is_in_the_past() {
        let cutoff = resolve_cutoff(&args(7)).unwrap();
        let now = Utc::now();
        assert!(cutoff < now);
        assert!(cutoff > now - Duration::days(8));
        assert!(cutoff <= now - Duration::days(7) + Duration::minutes(1));
    }

    #[test]
    fn annotation_backups_are_one_file_per_dataset_and_source() {
        let d1 = DatasetId("d1".to_owned());
        let d2 = DatasetId("d2".to_owned());
        let s1 = SourceId("s1".to_owned());
        let s2 = SourceId("s2".to_owned());

        // A dataset with two sources produces a distinct file per source, so a
        // multi-source dataset never collapses into one mixed file that couldn't be
        // restored to the right source.
        assert_ne!(
            annotation_backup_file(&d1, &s1),
            annotation_backup_file(&d1, &s2)
        );
        // A source shared by two datasets is backed up once per dataset (a comment
        // can be reviewed differently in each).
        assert_ne!(
            annotation_backup_file(&d1, &s1),
            annotation_backup_file(&d2, &s1)
        );

        // Layout is keyed by source id under a per-dataset directory, and the file
        // name matches the per-source deletion-set comment backup — so each
        // annotation file restores with one `re create comments --source --dataset`.
        assert_eq!(annotation_backup_file(&d1, &s1), "annotations/d1/s1.jsonl");
        assert_eq!(comment_backup_file(&s1), "deleted-comments/s1.jsonl");
        assert_eq!(
            email_backup_file(&BucketId("b1".to_owned())),
            "deleted-emails/b1.jsonl"
        );
        assert!(annotation_backup_file(&d1, &s1).ends_with("/s1.jsonl"));
        assert!(comment_backup_file(&s1).ends_with("/s1.jsonl"));
    }

    #[test]
    fn cutoff_older_than_zero_days_is_about_now() {
        // The degenerate `--older-than-days=0` means "everything up to now".
        let before = Utc::now();
        let cutoff = resolve_cutoff(&args(0)).unwrap();
        let after = Utc::now();
        assert!(cutoff >= before && cutoff <= after);
    }

    fn sample_cutoff() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn confirm_prompt_default_is_whole_bucket_and_keeps_annotated() {
        let datasets = vec!["owner/a".to_owned(), "owner/b".to_owned()];
        let prompt = confirm_prompt(&datasets, 3, 2, sample_cutoff(), false, None);
        assert!(prompt.contains("2 dataset(s): owner/a, owner/b"));
        assert!(prompt.contains("3 source(s) and 2 bucket(s)"));
        // Whole-bucket email wording, no mailbox restriction, no annotated-delete note.
        assert!(prompt.contains("whole-bucket by age"));
        assert!(!prompt.contains("restricted to mailbox"));
        assert!(!prompt.contains("Annotated comments will be deleted too"));
        // The unconditional data-loss caveats must always be present, so a reword
        // can't silently drop them.
        assert!(prompt.contains("attachment content is not backed up"));
        assert!(prompt.contains("a project you can't access"));
        assert!(prompt.contains("out of scope"));
        assert!(prompt.contains("their comments are NOT deleted"));
        assert!(prompt.contains("restore is manual"));
    }

    #[test]
    fn confirm_prompt_mailbox_adds_restriction_and_caveat() {
        let datasets = vec!["owner/a".to_owned()];
        let prompt = confirm_prompt(&datasets, 1, 1, sample_cutoff(), false, Some("sales@x.com"));
        assert!(prompt.contains("in mailbox `sales@x.com`"));
        assert!(prompt.contains("Email deletion is restricted to mailbox `sales@x.com`"));
        // The mailbox-only comment-matching caveat is present.
        assert!(prompt.contains("`Mailbox ID` user property"));
        assert!(!prompt.contains("whole-bucket by age"));
    }

    #[test]
    fn confirm_prompt_include_annotated_adds_warning() {
        let datasets = vec!["owner/a".to_owned()];
        let prompt = confirm_prompt(&datasets, 1, 0, sample_cutoff(), true, None);
        assert!(prompt.contains("Annotated comments will be deleted too"));
        // Pin the cross-project data-loss sub-clause with its plural-specific wording,
        // so it isn't satisfied by the always-on singular caveat elsewhere in the prompt.
        assert!(prompt.contains("datasets you can't access"));
        assert!(prompt.contains("annotations are NOT backed up"));
        // Flag-gated: absent when --include-annotated is off.
        let without = confirm_prompt(&datasets, 1, 0, sample_cutoff(), false, None);
        assert!(!without.contains("Annotated comments will be deleted too"));
    }

    #[test]
    fn backup_dir_is_write_once() {
        let root = temp_path("dir");
        let dir = create_backup_dir(&root, "run-1").unwrap();
        assert!(dir.exists());
        assert!(
            create_backup_dir(&root, "run-1").is_err(),
            "must refuse to reuse"
        );
        assert!(
            create_backup_dir(&root, "run-2").is_ok(),
            "a fresh run id is fine"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn for_each_verified_batch_round_trips_in_bounded_batches() {
        let path = temp_path("rt");
        let ids: Vec<String> = (0..70).map(|i| format!("e{i}")).collect();
        let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        let (count, crc32) = write_id_backup(&path, &id_refs);
        assert_eq!(count, 70);

        let expected = backup_file(count, crc32);
        let mut seen = Vec::new();
        let mut batch_sizes = Vec::new();
        for_each_verified_batch(&path, &expected, email_id, |batch: &[EmailId]| {
            batch_sizes.push(batch.len());
            seen.extend(batch.iter().map(|id| id.0.clone()));
            Ok(())
        })
        .unwrap();

        assert_eq!(seen, ids, "every id is read back in order");
        assert!(batch_sizes.len() >= 3, "70 ids span multiple batches");
        assert!(
            batch_sizes.iter().all(|&n| n <= DELETION_BATCH_SIZE),
            "no batch exceeds the API cap"
        );
        fs::remove_file(&path).ok();
    }

    #[test]
    fn for_each_verified_batch_extracts_nested_comment_id() {
        let path = temp_path("cid");
        let mut writer = BackupWriter::create(&path).unwrap();
        writer
            .write_record(&serde_json::json!({"comment": {"id": "c1", "extra": 1}}))
            .unwrap();
        writer
            .write_record(&serde_json::json!({"comment": {"id": "c2"}}))
            .unwrap();
        let (count, crc32) = writer.finish().unwrap();

        let expected = backup_file(count, crc32);
        let mut seen = Vec::new();
        for_each_verified_batch(&path, &expected, comment_id, |batch: &[CommentId]| {
            seen.extend(batch.iter().map(|id| id.0.clone()));
            Ok(())
        })
        .unwrap();
        assert_eq!(seen, vec!["c1".to_owned(), "c2".to_owned()]);
        fs::remove_file(&path).ok();
    }

    #[test]
    fn for_each_verified_batch_rejects_crc_mismatch() {
        let path = temp_path("crc");
        let (count, crc32) = write_id_backup(&path, &["a", "b"]);
        let tampered = backup_file(count, crc32.wrapping_add(1));
        assert!(
            for_each_verified_batch(&path, &tampered, email_id, |_: &[EmailId]| Ok(())).is_err()
        );
        fs::remove_file(&path).ok();
    }

    #[test]
    fn for_each_verified_batch_rejects_count_mismatch() {
        let path = temp_path("count");
        let (count, crc32) = write_id_backup(&path, &["a", "b"]);
        let tampered = backup_file(count + 1, crc32);
        assert!(
            for_each_verified_batch(&path, &tampered, email_id, |_: &[EmailId]| Ok(())).is_err()
        );
        fs::remove_file(&path).ok();
    }

    #[test]
    fn for_each_verified_batch_rejects_corrupt_line() {
        let path = temp_path("corrupt");
        {
            let mut file = File::create(&path).unwrap();
            file.write_all(b"{\"id\":\"ok\"}\nnot json\n").unwrap();
        }
        // CRC/count match the file; the corrupt line must still abort the read.
        let (count, crc32) = crc_and_count(&path).unwrap();
        let expected = backup_file(count, crc32);
        assert!(
            for_each_verified_batch(&path, &expected, email_id, |_: &[EmailId]| Ok(())).is_err()
        );
        fs::remove_file(&path).ok();
    }

    #[test]
    fn verify_backup_checks_crc_and_count() {
        let path = temp_path("verify");
        let (count, crc32) = write_id_backup(&path, &["x", "y", "z"]);
        assert_eq!(count, 3);
        assert!(verify_backup(&path, &backup_file(count, crc32)).is_ok());
        assert!(verify_backup(&path, &backup_file(count, crc32.wrapping_add(1))).is_err());
        assert!(verify_backup(&path, &backup_file(count + 1, crc32)).is_err());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn crc_count_and_uids_collects_uids_and_matches_checksum() {
        let path = temp_path("uids");
        let mut writer = BackupWriter::create(&path).unwrap();
        // Annotation backups are AnnotatedComment lines; we only read comment.uid.
        writer
            .write_record(&serde_json::json!({"comment": {"id": "c1", "uid": "src.c1"}}))
            .unwrap();
        writer
            .write_record(&serde_json::json!({"comment": {"id": "c2", "uid": "src.c2"}}))
            .unwrap();
        let (count, crc32) = writer.finish().unwrap();

        let mut reviewed = HashSet::new();
        // Same count and checksum as crc_and_count, plus the uids are collected.
        assert_eq!(
            crc_count_and_uids(&path, &mut reviewed).unwrap(),
            (count, crc32)
        );
        assert_eq!(crc_and_count(&path).unwrap(), (count, crc32));
        assert!(reviewed.contains(&CommentUid("src.c1".to_owned())));
        assert!(reviewed.contains(&CommentUid("src.c2".to_owned())));
        assert_eq!(reviewed.len(), 2);

        // A line missing comment.uid is rejected, like other corrupt lines.
        let bad = temp_path("uids-bad");
        let (_, _) = write_id_backup(&bad, &["x"]); // {"id":"x"} has no comment.uid
        assert!(crc_count_and_uids(&bad, &mut HashSet::new()).is_err());

        fs::remove_file(&path).ok();
        fs::remove_file(&bad).ok();
    }

    /// Build a `Comment` with an optional "Mailbox ID" string user property, going
    /// through the wire format (so the `string:` prefix is exercised by the model).
    fn comment_with_mailbox(mailbox: Option<&str>) -> Comment {
        let mut properties = serde_json::Map::new();
        if let Some(mailbox) = mailbox {
            properties.insert("string:Mailbox ID".to_owned(), serde_json::json!(mailbox));
        }
        serde_json::from_value(serde_json::json!({
            "id": "c1",
            "uid": "src.c1",
            "timestamp": "2024-01-01T00:00:00Z",
            "messages": [],
            "user_properties": properties,
            "created_at": "2024-01-01T00:00:00Z",
        }))
        .unwrap()
    }

    #[test]
    fn comment_in_mailbox_matches_property_case_insensitively() {
        let comment = comment_with_mailbox(Some("Sales@Example.com"));
        assert!(comment_in_mailbox(&comment, "Sales@Example.com"));
        // Case-insensitive: parse-email's Normalized mode lowercases the stored value.
        assert!(comment_in_mailbox(&comment, "sales@example.com"));
        assert!(!comment_in_mailbox(&comment, "support@example.com"));
    }

    #[test]
    fn comment_in_mailbox_is_false_without_the_property() {
        // No "Mailbox ID" property (mailbox-id injection disabled) → never matched,
        // so a mailbox-scoped run never deletes the comment.
        let comment = comment_with_mailbox(None);
        assert!(!comment_in_mailbox(&comment, "sales@example.com"));
    }

    #[test]
    fn crc_and_count_matches_backup_writer() {
        let path = temp_path("match");
        let (count, crc32) = write_id_backup(&path, &["one", "two"]);
        // Reading the file back must reproduce the writer's count and checksum.
        assert_eq!(crc_and_count(&path).unwrap(), (count, crc32));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn comment_in_mailbox_is_false_for_number_typed_property() {
        // A number-typed "Mailbox ID" deserializes to PropertyValue::Number, which the
        // String-only match rejects — so the comment is KEPT (the fail-safe direction).
        // A future change that started matching numbers would flip this and fail.
        let comment: Comment = serde_json::from_value(serde_json::json!({
            "id": "c1",
            "uid": "src.c1",
            "timestamp": "2024-01-01T00:00:00Z",
            "messages": [],
            "user_properties": { "number:Mailbox ID": 42 },
            "created_at": "2024-01-01T00:00:00Z",
        }))
        .unwrap();
        assert!(!comment_in_mailbox(&comment, "42"));
    }

    #[test]
    fn validate_args_requires_at_least_one_dataset() {
        let err = validate_args(&args(7)).unwrap_err();
        assert!(err.to_string().contains("At least one dataset"), "{err}");

        let mut with_dataset = args(7);
        with_dataset.datasets = vec!["owner/name".parse().unwrap()];
        assert!(validate_args(&with_dataset).is_ok());
    }

    #[test]
    fn cutoff_with_huge_older_than_days_errors_instead_of_panicking() {
        // chrono's `DateTime - Duration` panics on overflow; a huge --older-than-days
        // must instead return a clean error.
        assert!(resolve_cutoff(&args(u32::MAX)).is_err());
    }

    #[test]
    fn decide_accepts_yes_and_aborts_on_no() {
        assert!(decide(true).is_ok());
        let err = decide(false).unwrap_err();
        assert!(err.to_string().contains("Prune aborted by user."), "{err}");
    }

    #[test]
    fn for_each_verified_batch_never_invokes_on_batch_for_empty_backup() {
        // An empty deletion set must issue ZERO delete batches — an empty id list to a
        // DELETE endpoint is the classic "delete everything" footgun.
        let path = temp_path("empty");
        let (count, crc32) = write_id_backup(&path, &[]);
        assert_eq!(count, 0);
        let expected = backup_file(count, crc32);
        let mut calls = 0;
        for_each_verified_batch(&path, &expected, email_id, |_: &[EmailId]| {
            calls += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(calls, 0, "no delete batch issued for an empty backup");
        fs::remove_file(&path).ok();
    }

    /// Build a `Manifest` referencing the given backup files. Only the fields the
    /// verify/delete passes read need be meaningful.
    fn manifest_with(
        comment_backups: Vec<BackupFile>,
        email_backups: Vec<BackupFile>,
        annotation_backups: Vec<BackupFile>,
    ) -> Manifest {
        Manifest {
            run_id: "run".to_owned(),
            cutoff: sample_cutoff(),
            include_annotated: false,
            mailbox: None,
            datasets: vec![],
            annotation_backups,
            comment_backups,
            email_backups,
            comment_count: 0,
            email_count: 0,
        }
    }

    /// Write a comment-shaped backup (`{"comment":{"id":...}}` lines) at
    /// `dir/<name>.jsonl` and return its manifest entry (resource == name).
    fn write_comment_backup(dir: &Path, name: &str, ids: &[&str]) -> BackupFile {
        let file = format!("{name}.jsonl");
        let mut writer = BackupWriter::create(&dir.join(&file)).unwrap();
        for id in ids {
            writer
                .write_record(&serde_json::json!({ "comment": { "id": id } }))
                .unwrap();
        }
        let (count, crc32) = writer.finish().unwrap();
        BackupFile {
            resource: name.to_owned(),
            file,
            count,
            crc32,
        }
    }

    /// Write an email-shaped backup (`{"id":...}` lines) at `dir/<name>.jsonl`.
    fn write_email_backup(dir: &Path, name: &str, ids: &[&str]) -> BackupFile {
        let file = format!("{name}.jsonl");
        let mut writer = BackupWriter::create(&dir.join(&file)).unwrap();
        for id in ids {
            writer
                .write_record(&serde_json::json!({ "id": id }))
                .unwrap();
        }
        let (count, crc32) = writer.finish().unwrap();
        BackupFile {
            resource: name.to_owned(),
            file,
            count,
            crc32,
        }
    }

    /// A copy of `backup` whose recorded CRC no longer matches its file — standing in
    /// for an on-disk corruption that happened after the manifest was written.
    fn with_bad_crc(mut backup: BackupFile) -> BackupFile {
        backup.crc32 = backup.crc32.wrapping_add(1);
        backup
    }

    #[test]
    fn verify_then_delete_deletes_every_backed_up_id_when_intact() {
        // Positive control: when every backup verifies, the deleter receives every id,
        // attributed to its resource — so the zero-delete corruption tests below are
        // not passing vacuously.
        let dir = temp_path("vtd-ok");
        fs::create_dir_all(&dir).unwrap();
        let manifest = manifest_with(
            vec![
                write_comment_backup(&dir, "src-a", &["c1", "c2"]),
                write_comment_backup(&dir, "src-b", &["c3"]),
            ],
            vec![write_email_backup(&dir, "bucket-a", &["e1", "e2"])],
            vec![],
        );

        let comments = RefCell::new(Vec::new());
        let emails = RefCell::new(Vec::new());
        verify_then_delete(
            &dir,
            &manifest,
            false,
            |resource: &str, ids: &[CommentId]| {
                comments.borrow_mut().push((resource.to_owned(), ids.len()));
                Ok(())
            },
            |resource: &str, ids: &[EmailId]| {
                emails.borrow_mut().push((resource.to_owned(), ids.len()));
                Ok(())
            },
        )
        .unwrap();

        let comment_total: usize = comments.borrow().iter().map(|(_, n)| n).sum();
        let email_total: usize = emails.borrow().iter().map(|(_, n)| n).sum();
        assert_eq!(
            comment_total,
            3,
            "every comment id deleted: {:?}",
            comments.borrow()
        );
        assert_eq!(
            email_total,
            2,
            "every email id deleted: {:?}",
            emails.borrow()
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn verify_then_delete_aborts_with_zero_deletes_when_a_later_comment_backup_is_corrupt() {
        // Two comment backups; corrupt the SECOND. The run must abort in PASS 1 with the
        // first (valid) backup never deleted — proving verify-ALL-before-delete-ANY, not
        // a per-file interleave that would have deleted the first already.
        let dir = temp_path("vtd-bad-comment");
        fs::create_dir_all(&dir).unwrap();
        let manifest = manifest_with(
            vec![
                write_comment_backup(&dir, "src-a", &["c1", "c2"]),
                with_bad_crc(write_comment_backup(&dir, "src-b", &["c3"])),
            ],
            vec![],
            vec![],
        );

        let deletes = RefCell::new(0usize);
        let result = verify_then_delete(
            &dir,
            &manifest,
            false,
            |_: &str, ids: &[CommentId]| {
                *deletes.borrow_mut() += ids.len();
                Ok(())
            },
            |_: &str, _: &[EmailId]| Ok(()),
        );
        assert!(result.is_err(), "a corrupt backup must abort the run");
        assert_eq!(
            *deletes.borrow(),
            0,
            "no comment deleted when any backup fails verification"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn verify_then_delete_corrupt_email_backup_blocks_comment_deletion() {
        // Cross-resource: a VALID, non-empty comment backup plus a corrupt EMAIL backup.
        // The comment backup verifies first, but email verification (later in PASS 1)
        // fails, so no comment is deleted.
        let dir = temp_path("vtd-bad-email");
        fs::create_dir_all(&dir).unwrap();
        let manifest = manifest_with(
            vec![write_comment_backup(&dir, "src-a", &["c1", "c2"])],
            vec![with_bad_crc(write_email_backup(&dir, "bucket-a", &["e1"]))],
            vec![],
        );

        let comments = RefCell::new(0usize);
        let emails = RefCell::new(0usize);
        let result = verify_then_delete(
            &dir,
            &manifest,
            false,
            |_: &str, ids: &[CommentId]| {
                *comments.borrow_mut() += ids.len();
                Ok(())
            },
            |_: &str, ids: &[EmailId]| {
                *emails.borrow_mut() += ids.len();
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(
            *comments.borrow(),
            0,
            "corrupt email backup must block comment deletion"
        );
        assert_eq!(*emails.borrow(), 0);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn verify_then_delete_corrupt_annotation_backup_blocks_all_deletion() {
        // Annotation backups are verified (though never deleted from); a corrupt one
        // must still abort before any comment or email deletion.
        let dir = temp_path("vtd-bad-annotation");
        fs::create_dir_all(&dir).unwrap();
        let manifest = manifest_with(
            vec![write_comment_backup(&dir, "src-a", &["c1", "c2"])],
            vec![write_email_backup(&dir, "bucket-a", &["e1"])],
            vec![with_bad_crc(write_comment_backup(
                &dir,
                "annotations-src-a",
                &["c1"],
            ))],
        );

        let comments = RefCell::new(0usize);
        let emails = RefCell::new(0usize);
        let result = verify_then_delete(
            &dir,
            &manifest,
            false,
            |_: &str, ids: &[CommentId]| {
                *comments.borrow_mut() += ids.len();
                Ok(())
            },
            |_: &str, ids: &[EmailId]| {
                *emails.borrow_mut() += ids.len();
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(*comments.borrow(), 0);
        assert_eq!(*emails.borrow(), 0);
        fs::remove_dir_all(&dir).ok();
    }
}
