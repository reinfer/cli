use crate::{TestCli, TestSource};
use reinfer_client::NewAnnotatedComment;

const SAMPLE_BASIC: &str = include_str!("./samples/basic.jsonl");

#[test]
fn test_comments_lifecycle() {
    let annotated_comments: Vec<NewAnnotatedComment> = SAMPLE_BASIC
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
        SAMPLE_BASIC.as_bytes(),
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
