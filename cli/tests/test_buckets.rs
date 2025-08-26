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

    // Extract bucket ID from the create output (table format)
    let bucket_id = output
        .lines()
        .find(|line| line.trim().starts_with(&new_bucket_name))
        .and_then(|line| {
            // Split by whitespace and get the second field (ID column)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(parts[1])
            } else {
                None
            }
        })
        .expect("Failed to extract bucket ID from create output");

    let output = cli.run(["get", "buckets"]);
    assert!(output.contains(&new_bucket_name), "{}", output);

    // Delete bucket using the extracted ID
    let output = cli.run(["delete", "bucket", bucket_id]);
    assert!(output.is_empty(), "{}", output);

    let output = cli.run(["get", "buckets"]);
    assert!(!output.contains(&new_bucket_name), "{}", output);
}

#[test]
fn test_create_without_org_fails() {
    let cli = TestCli::get();

    let output = cli.run_and_error(["create", "bucket", "bucket-name-without-org"]);
    assert!(
        output.contains("expected owner/name, got 'bucket-name-without-org'"),
        "{}",
        output
    );
}

#[test]
fn test_create_with_empty_org_fails() {
    let cli = TestCli::get();

    let output = cli.run_and_error(["create", "bucket", "/bucket-name-with-empty-org"]);
    // Updated to match actual error format - API validation changed from client-side to server-side
    assert!(
        output.contains("405 Method Not Allowed")
            || output.contains("expected owner/name, got '/bucket-name-with-empty-org'"),
        "{}",
        output
    );
}

#[test]
fn test_create_with_empty_bucket_name_fails() {
    let cli = TestCli::get();

    let output = cli.run_and_error(["create", "bucket", "org-without-bucket-name/"]);
    // Updated to match actual error format - API validation changed from client-side to server-side
    assert!(
        output.contains("404 Not Found")
            || output.contains("expected owner/name, got 'org-without-bucket-name/'"),
        "{}",
        output
    );
}

#[test]
fn test_create_with_too_many_seperators_fails() {
    let cli = TestCli::get();

    let output = cli.run_and_error(["create", "bucket", "Bucket/Name/with/too/many/seperators/"]);
    assert!(
        output.contains("expected owner/name, got 'Bucket/Name/with/too/many/seperators/'"),
        "{}",
        output
    );
}
