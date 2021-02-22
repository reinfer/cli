use crate::{TestCli, TestSource};
use reinfer_client::NewAnnotatedComment;

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
