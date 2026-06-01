use crate::TestCli;
use uuid::Uuid;

#[test]
fn test_bucket_lifecycle() {
    let cli = TestCli::get();
    let owner = TestCli::project();

    let new_bucket_name = format!("{}/test-source-{}", owner, Uuid::new_v4());

    // Create bucket
    let output = cli.run(["create", "bucket", &new_bucket_name]);
    assert!(output.contains(&new_bucket_name), "{}", output);

    let output = cli.run(["get", "buckets"]);
    assert!(output.contains(&new_bucket_name), "{}", output);

    // Deleting one comment reduces the comment count in the source
    let output = cli.run(["delete", "bucket", &new_bucket_name]);
    assert!(output.is_empty(), "{}", output);

    let output = cli.run(["get", "buckets"]);
    assert!(!output.contains(&new_bucket_name), "{}", output);
}

#[test]
fn test_get_emails_filter_by_mailbox_and_timerange() {
    let cli = TestCli::get();
    let owner = TestCli::project();

    let bucket = format!("{}/test-emails-{}", owner, Uuid::new_v4());
    cli.run(["create", "bucket", &bucket]);

    // Four emails across two mailboxes and a spread of timestamps.
    let emails = [
        ("alice-1", "alice@reinfer.io", "2020-01-01T00:00:00Z"),
        ("alice-2", "alice@reinfer.io", "2020-02-01T00:00:00Z"),
        ("bob-1", "bob@reinfer.io", "2020-01-15T00:00:00Z"),
        ("bob-2", "bob@reinfer.io", "2020-03-01T00:00:00Z"),
    ];
    let jsonl = emails
        .iter()
        .map(|(id, mailbox, timestamp)| {
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
        })
        .collect::<Vec<_>>()
        .join("\n");

    cli.run_with_stdin(["create", "emails", "-y", "-b", &bucket], jsonl.as_bytes());

    // Extract a sorted list of the given field across the returned JSONL.
    let field = |output: &str, key: &str| -> Vec<String> {
        let mut values: Vec<String> = output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<serde_json::Value>(line).unwrap()[key]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        values.sort();
        values
    };

    // No filter returns every email via the listing endpoint (the `None`
    // branch of `download_emails`).
    let all = field(&cli.run(["get", "emails", &bucket]), "id");
    assert_eq!(
        all,
        vec!["alice-1", "alice-2", "bob-1", "bob-2"],
        "unfiltered get should return all emails"
    );

    // Filter by mailbox name (exact match).
    let alice = field(
        &cli.run(["get", "emails", &bucket, "--mailbox", "alice@reinfer.io"]),
        "id",
    );
    assert_eq!(alice, vec!["alice-1", "alice-2"], "alice mailbox filter");

    // Filter by timerange [from inclusive, to exclusive): catches alice-2 and bob-1.
    let in_range = field(
        &cli.run([
            "get",
            "emails",
            &bucket,
            "--from-timestamp",
            "2020-01-10T00:00:00Z",
            "--to-timestamp",
            "2020-02-15T00:00:00Z",
        ]),
        "id",
    );
    assert_eq!(in_range, vec!["alice-2", "bob-1"], "timerange filter");

    // Combined mailbox + timerange narrows to only alice-2.
    let combined = field(
        &cli.run([
            "get",
            "emails",
            &bucket,
            "--mailbox",
            "alice@reinfer.io",
            "--from-timestamp",
            "2020-01-10T00:00:00Z",
            "--to-timestamp",
            "2020-02-15T00:00:00Z",
        ]),
        "id",
    );
    assert_eq!(combined, vec!["alice-2"], "mailbox + timerange filter");

    // `from` after `to` is rejected client-side before hitting the API.
    let error = cli.run_and_error([
        "get",
        "emails",
        &bucket,
        "--from-timestamp",
        "2020-02-01T00:00:00Z",
        "--to-timestamp",
        "2020-01-01T00:00:00Z",
    ]);
    assert!(error.contains("must be less than or equal to"), "{}", error);

    cli.run(["delete", "bucket", &bucket]);
}

#[test]
fn test_delete_emails() {
    let cli = TestCli::get();
    let owner = TestCli::project();

    let bucket = format!("{}/test-delete-emails-{}", owner, Uuid::new_v4());
    cli.run(["create", "bucket", &bucket]);

    // Three emails in the bucket.
    let emails = [
        ("keep", "alice@reinfer.io", "2020-01-01T00:00:00Z"),
        ("delete-1", "bob@reinfer.io", "2020-02-01T00:00:00Z"),
        ("delete-2", "carol@reinfer.io", "2020-03-01T00:00:00Z"),
    ];
    let jsonl = emails
        .iter()
        .map(|(id, mailbox, timestamp)| {
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
        })
        .collect::<Vec<_>>()
        .join("\n");
    cli.run_with_stdin(["create", "emails", "-y", "-b", &bucket], jsonl.as_bytes());

    // Sorted list of email ids currently in the bucket.
    let ids = |output: &str| -> Vec<String> {
        let mut values: Vec<String> = output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<serde_json::Value>(line).unwrap()["id"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        values.sort();
        values
    };

    assert_eq!(
        ids(&cli.run(["get", "emails", &bucket])),
        vec!["delete-1", "delete-2", "keep"],
        "all three emails should be present before deletion"
    );

    // Delete two of the three by id.
    let output = cli.run(["delete", "emails", "-b", &bucket, "delete-1", "delete-2"]);
    assert!(output.is_empty(), "{}", output);

    assert_eq!(
        ids(&cli.run(["get", "emails", &bucket])),
        vec!["keep"],
        "only the un-deleted email should remain"
    );

    // Deletion is idempotent: deleting an already-deleted / missing id succeeds.
    let output = cli.run(["delete", "emails", "-b", &bucket, "delete-1", "does-not-exist"]);
    assert!(output.is_empty(), "{}", output);
    assert_eq!(
        ids(&cli.run(["get", "emails", &bucket])),
        vec!["keep"],
        "idempotent delete of missing ids should not affect remaining emails"
    );

    cli.run(["delete", "bucket", &bucket]);
}

#[test]
fn test_create_without_org_fails() {
    let cli = TestCli::get();

    let output = cli.run_and_error(["create", "bucket", "bucket-name-without-org"]);
    assert!(
        output.contains("Expected <owner>/<name>, got: bucket-name-without-org"),
        "{}",
        output
    );
}

#[test]
fn test_create_with_empty_org_fails() {
    let cli = TestCli::get();

    let output = cli.run_and_error(["create", "bucket", "/bucket-name-with-empty-org"]);
    assert!(
        output.contains("Expected <owner>/<name>, got: /bucket-name-with-empty-org"),
        "{}",
        output
    );
}

#[test]
fn test_create_with_empty_bucket_name_fails() {
    let cli = TestCli::get();

    let output = cli.run_and_error(["create", "bucket", "org-without-bucket-name/"]);
    assert!(
        output.contains("Expected <owner>/<name>, got: org-without-bucket-name/"),
        "{}",
        output
    );
}

#[test]
fn test_create_with_too_many_seperators_fails() {
    let cli = TestCli::get();

    let output = cli.run_and_error(["create", "bucket", "Bucket/Name/with/too/many/seperators/"]);
    assert!(
        output.contains("Expected <owner>/<name>, got: Bucket/Name/with/too/many/seperators"),
        "{}",
        output
    );
}
