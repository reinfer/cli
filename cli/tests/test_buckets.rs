use crate::TestCli;
use uuid::Uuid;

#[test]
fn test_bucket_lifecycle() {
    let cli = TestCli::get();
    let owner = TestCli::organisation();

    let new_bucket_name = format!("{}/test-source-{}", owner, Uuid::new_v4());

    // Create bucket
    let output = cli.run(&["create", "bucket", &new_bucket_name]);
    assert!(output.is_empty());

    let output = cli.run(&["get", "buckets"]);
    assert!(output.contains(&new_bucket_name));

    // Deleting one comment reduces the comment count in the source
    let output = cli.run(&["delete", "bucket", &new_bucket_name]);
    assert!(output.is_empty());

    let output = cli.run(&["get", "buckets"]);
    assert!(!output.contains(&new_bucket_name));
}

#[test]
fn test_bucket_with_invalid_transform_tag_fails() {
    let cli = TestCli::get();
    let owner = TestCli::organisation();

    let new_bucket_name = format!("{}/test-source-{}", owner, Uuid::new_v4());

    let output = cli.run_and_error(&[
        "create",
        "bucket",
        &new_bucket_name,
        "--transform-tag",
        "not-a-valid-transform-tag.0.ABCDEFGH",
    ]);
    assert!(
        output.contains(
        "422 Unprocessable Entity: The value 'not-a-valid-transform-tag.0.ABCDEFGH' is not a valid transform tag.")
    );
}
