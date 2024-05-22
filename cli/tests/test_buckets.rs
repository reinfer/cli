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
