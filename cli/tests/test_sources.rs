use crate::common::TestCli;
use pretty_assertions::assert_eq;
use reinfer_client::Source;
use uuid::Uuid;

pub struct TestSource {
    full_name: String,
    sep_index: usize,
}

impl TestSource {
    pub fn new() -> Self {
        let cli = TestCli::get();
        let project_name = TestCli::project();
        let full_name = format!("{}/test-source-{}", project_name, Uuid::new_v4());
        let sep_index = project_name.len();

        let output = cli.run(&["create", "source", &full_name]);
        assert!(output.contains(&full_name));

        Self {
            full_name,
            sep_index,
        }
    }

    pub fn new_args(args: &[&str]) -> Self {
        let cli = TestCli::get();
        let project_name = TestCli::project();
        let full_name = format!("{}/test-source-{}", project_name, Uuid::new_v4());
        let sep_index = project_name.len();

        let output = cli.run(["create", "source", &full_name].iter().chain(args));
        assert!(output.contains(&full_name));

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

    pub fn get(&self) -> Source {
        let output = TestCli::get().run(&["--output=json", "get", "sources", self.identifier()]);
        serde_json::from_str::<Source>(&output).unwrap()
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
fn test_create_update_source_custom() {
    let cli = TestCli::get();
    let source = TestSource::new_args(&[
        "--title=some title",
        "--description=some description",
        "--language=de",
        "--should-translate=true",
    ]);

    /// A subset of source fields that we can easily check for equality accross
    #[derive(PartialEq, Eq, Debug)]
    struct SourceInfo {
        owner: String,
        name: String,
        title: String,
        description: String,
        language: String,
        should_translate: bool,
        kind: String,
    }

    impl From<Source> for SourceInfo {
        fn from(source: Source) -> SourceInfo {
            SourceInfo {
                owner: source.owner.0,
                name: source.name.0,
                title: source.title,
                description: source.description,
                language: source.language,
                should_translate: source.should_translate,
                kind: source.kind.to_string(),
            }
        }
    }

    let get_source_info = || -> SourceInfo {
        let output = cli.run(&["--output=json", "get", "sources", source.identifier()]);
        serde_json::from_str::<Source>(&output).unwrap().into()
    };

    let mut expected_source_info = SourceInfo {
        owner: source.owner().to_owned(),
        name: source.name().to_owned(),
        title: "some title".to_owned(),
        description: "some description".to_owned(),
        language: "de".to_owned(),
        should_translate: true,
        kind: "unknown".to_owned(),
    };
    assert_eq!(get_source_info(), expected_source_info);

    // An empty update should be fine
    cli.run(&["update", "source", source.identifier()]);
    assert_eq!(get_source_info(), expected_source_info);

    // Partial update
    cli.run(&[
        "update",
        "source",
        "--title=updated title",
        source.identifier(),
    ]);
    expected_source_info.title = "updated title".to_owned();
    assert_eq!(get_source_info(), expected_source_info);

    // Should be able to update all fields
    cli.run(&[
        "update",
        "source",
        "--title=updated title",
        "--description=updated description",
        "--should-translate=false",
        source.identifier(),
    ]);
    expected_source_info.title = "updated title".to_owned();
    expected_source_info.description = "updated description".to_owned();
    expected_source_info.should_translate = false;
    assert_eq!(get_source_info(), expected_source_info);
}

#[test]
fn test_create_source_with_kind() {
    let cli = TestCli::get();
    let source = TestSource::new_args(&["--title=some title", "--kind=call"]);

    /// A subset of source fields that we can easily check for equality accross
    #[derive(PartialEq, Eq, Debug)]
    struct SourceInfo {
        owner: String,
        name: String,
        title: String,
        kind: String,
    }

    impl From<Source> for SourceInfo {
        fn from(source: Source) -> SourceInfo {
            SourceInfo {
                owner: source.owner.0,
                name: source.name.0,
                title: source.title,
                kind: source.kind.to_string(),
            }
        }
    }

    let get_source_info = || -> SourceInfo {
        let output = cli.run(&["--output=json", "get", "sources", source.identifier()]);
        serde_json::from_str::<Source>(&output).unwrap().into()
    };

    let expected_source_info = SourceInfo {
        owner: source.owner().to_owned(),
        name: source.name().to_owned(),
        title: "some title".to_owned(),
        kind: "call".to_owned(),
    };
    assert_eq!(get_source_info(), expected_source_info);
}

#[test]
fn test_create_source_requires_owner() {
    let cli = TestCli::get();

    let output = cli
        .command()
        .args(&["create", "source", "source-without-owner"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim(),
        "error: Invalid value for '<source-name>': Expected <owner>/<name> or a source id, got: source-without-owner"
    );
}
