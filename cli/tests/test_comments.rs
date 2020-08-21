use crate::{TestCli, TestSource};

const SAMPLE_BASIC: &str = include_str!("./samples/basic.jsonl");

#[test]
fn test_upload_comments() {
    let cli = TestCli::get();
    let source = TestSource::new();

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
}
