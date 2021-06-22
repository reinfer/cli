use crate::{TestCli, TestDataset, TestSource};
use chrono::DateTime;
use pretty_assertions::assert_eq;
use reinfer_client::{AnnotatedComment, NewAnnotatedComment};

#[test]
fn test_comments_lifecycle_basic() {
    const SAMPLE_BASIC: &str = include_str!("./samples/basic.jsonl");
    check_comments_lifecycle(SAMPLE_BASIC);
}

#[test]
fn test_comments_lifecycle_audio() {
    const SAMPLE_AUDIO: &str = include_str!("./samples/audio.jsonl");
    check_comments_lifecycle(SAMPLE_AUDIO);
}

fn check_comments_lifecycle(comments_str: &str) {
    let annotated_comments: Vec<NewAnnotatedComment> = comments_str
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .unwrap();

    let cli = TestCli::get();
    let source = TestSource::new();

    // Upload our test data
    let output = cli.run_with_stdin(
        &[
            "create",
            "comments",
            "--allow-duplicates",
            &format!("--source={}", source.identifier()),
        ],
        comments_str.as_bytes(),
    );
    assert!(output.is_empty());

    let output = cli.run(&["get", "comments", &source.identifier().to_string()]);
    assert_eq!(output.lines().count(), annotated_comments.len());

    // Test getting a comment by id to check the content matches
    let test_comment = annotated_comments.get(0).unwrap().comment.clone();
    let output = cli.run(&[
        "get",
        "comment",
        &format!("--source={}", source.identifier()),
        &test_comment.id.0,
    ]);
    let fetched_comment: AnnotatedComment =
        serde_json::from_str(&output).expect("invalid annotated comment fetched");
    assert_eq!(test_comment.id, fetched_comment.comment.id);
    assert_eq!(test_comment.messages, fetched_comment.comment.messages);
    assert_eq!(test_comment.timestamp, fetched_comment.comment.timestamp);
    assert_eq!(
        test_comment.user_properties,
        fetched_comment.comment.user_properties
    );

    // Deleting one comment reduces the comment count in the source
    let output = cli.run(&[
        "delete",
        "comments",
        &format!("--source={}", source.identifier()),
        &annotated_comments.get(0).unwrap().comment.id.0,
    ]);
    assert!(output.is_empty());

    let output = cli.run(&["get", "comments", &source.identifier().to_string()]);
    assert_eq!(output.lines().count(), annotated_comments.len() - 1);

    // Delete all ids
    let mut args = vec!["delete", "comments", "--source", source.identifier()];
    args.extend(
        annotated_comments
            .iter()
            .map(|annotated_comment| annotated_comment.comment.id.0.as_str()),
    );
    let output = cli.run(&args);
    assert!(output.is_empty());

    let output = cli.run(&["get", "comments", &source.identifier().to_string()]);
    assert!(output.is_empty());
}

#[test]
fn test_delete_comments_in_range() {
    let comments_str = include_str!("./samples/many.jsonl");
    let annotated_comments: Vec<NewAnnotatedComment> = comments_str
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .unwrap();
    let num_comments = annotated_comments.len();
    let num_annotated = annotated_comments
        .iter()
        .filter(|comment| comment.has_annotations())
        .count();

    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset1 = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);
    // let dataset2 = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);

    // Upload our test data
    let output = cli.run_with_stdin(
        &[
            "create",
            "comments",
            "--allow-duplicates",
            &format!("--source={}", source.identifier()),
            &format!("--dataset={}", dataset1.identifier()),
        ],
        comments_str.as_bytes(),
    );
    assert!(output.is_empty());

    let uploaded_all = cli.run(&["get", "comments", &source.identifier().to_string()]);
    assert_eq!(uploaded_all.lines().count(), num_comments);

    // Download annotated comments and check count
    let uploaded_annotated = cli.run(&[
        "get",
        "comments",
        "--reviewed-only",
        "true",
        "--dataset",
        dataset1.identifier(),
        source.identifier(),
    ]);
    assert_eq!(uploaded_annotated.lines().count(), num_annotated);

    // Delete comments in range. By default this should exclude annotated comments
    let from_timestamp_str = "2020-01-03T00:00:00Z";
    let from_timestamp = DateTime::parse_from_rfc3339(from_timestamp_str).unwrap();

    let to_timestamp_str = "2020-02-01T00:00:00Z";
    let to_timestamp = DateTime::parse_from_rfc3339(to_timestamp_str).unwrap();

    cli.run(&[
        "delete",
        "bulk",
        "--source",
        source.identifier(),
        "--from-timestamp",
        from_timestamp_str,
        "--to-timestamp",
        to_timestamp_str,
        "--include-annotated=false",
    ]);
    let num_deleted = annotated_comments
        .iter()
        .filter(|comment| {
            // N.B. to / from are inclusive
            !comment.has_annotations()
                && comment.comment.timestamp <= to_timestamp
                && comment.comment.timestamp >= from_timestamp
        })
        .count();

    // Get all comments and check counts
    let after_deleting_range = cli.run(&[
        "get",
        "comments",
        "--dataset",
        dataset1.identifier(),
        source.identifier(),
    ]);
    assert_eq!(
        after_deleting_range.lines().count(),
        num_comments - num_deleted
    );

    // Delete comments in source, excluding annotated comments
    cli.run(&[
        "delete",
        "bulk",
        "--source",
        source.identifier(),
        "--include-annotated=false",
    ]);

    // Get all comments and check that only annotated ones are left
    let after_deleting_unannotated = cli.run(&[
        "get",
        "comments",
        "--dataset",
        dataset1.identifier(),
        source.identifier(),
    ]);
    assert_eq!(after_deleting_unannotated.lines().count(), num_annotated);

    // Delete all comments
    cli.run(&[
        "delete",
        "bulk",
        &format!("--source={}", source.identifier()),
        "--include-annotated=true",
    ]);

    // Get all comments and check there are none left
    let after_deleting_all = cli.run(&[
        "get",
        "comments",
        "--dataset",
        dataset1.identifier(),
        source.identifier(),
    ]);
    assert_eq!(after_deleting_all.lines().count(), 0);
}
