use crate::common::TestCli;
use reinfer_client::Source;
use uuid::Uuid;

pub struct TestSource {
    full_name: String,
    sep_index: usize,
}

impl TestSource {
    pub fn new() -> Self {
        let cli = TestCli::get();
        let user = TestCli::organisation();
        let full_name = format!("{}/test-source-{}", user, Uuid::new_v4());
        let sep_index = user.len();

        let output = cli.run(&["create", "source", &full_name]);
        assert!(output.is_empty());

        Self {
            full_name,
            sep_index,
        }
    }

    pub fn new_args(args: &[&str]) -> Self {
        let cli = TestCli::get();
        let user = TestCli::organisation();
        let full_name = format!("{}/test-source-{}", user, Uuid::new_v4());
        let sep_index = user.len();

        let output = cli.run(["create", "source", &full_name].iter().chain(args));
        assert!(output.is_empty());

        Self {
            full_name,
            sep_index,
        }
    }

    pub fn identifier(&self) -> &str {
        &self.full_name
    }

    pub fn owner(&self) -> &str {
        &self.full_name[..self.sep_index]
    }

    pub fn name(&self) -> &str {
        &self.full_name[self.sep_index + 1..]
    }
}

impl Drop for TestSource {
    fn drop(&mut self) {
        let output = TestCli::get().run(&["delete", "source", self.identifier()]);
        assert!(output.is_empty());
    }
}

#[test]
fn test_test_source() {
    let cli = TestCli::get();
    let source = TestSource::new();

    let identifier = source.identifier().to_owned();

    let output = cli.run(&["get", "sources"]);
    assert!(output.contains(&identifier));

    drop(source);

    // RAII TestSource; should automatically clean up the temporary source on drop.
    let output = cli.run(&["get", "sources"]);
    assert!(!output.contains(&identifier));
}

#[test]
fn test_list_multiple_sources() {
    let cli = TestCli::get();
    let source1 = TestSource::new();
    let source2 = TestSource::new();

    let output = cli.run(&["get", "sources"]);
    assert!(output.contains(source1.identifier()));
    assert!(output.contains(source2.identifier()));

    let output = cli.run(&["get", "sources", source1.identifier()]);
    assert!(output.contains(source1.identifier()));
    assert!(!output.contains(source2.identifier()));

    let output = cli.run(&["get", "sources", source2.identifier()]);
    assert!(!output.contains(source1.identifier()));
    assert!(output.contains(source2.identifier()));
}

#[test]
fn test_create_source_custom() {
    let cli = TestCli::get();
    let source = TestSource::new_args(&[
        "--title=some title",
        "--description=some description",
        "--language=de",
        "--should-translate=true",
    ]);

    let output = cli.run(&["get", "sources", source.identifier(), "--output=json"]);
    let source_info: Source = serde_json::from_str(&output).unwrap();

    assert_eq!(&source_info.owner.0, source.owner());
    assert_eq!(&source_info.name.0, source.name());
    assert_eq!(source_info.title, "some title");
    assert_eq!(source_info.description, "some description");
    assert_eq!(source_info.language, "de");
    assert_eq!(source_info.should_translate, true);
}
