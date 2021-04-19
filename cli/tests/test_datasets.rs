use reinfer_client::{Dataset, Source};
use uuid::Uuid;

use crate::{TestCli, TestSource};

pub struct TestDataset {
    full_name: String,
    sep_index: usize,
}

impl TestDataset {
    pub fn new() -> Self {
        let cli = TestCli::get();
        let user = TestCli::organisation();
        let full_name = format!("{}/test-dataset-{}", user, Uuid::new_v4());
        let sep_index = user.len();

        let output = cli.run(&["create", "dataset", &full_name]);
        assert!(output.is_empty());

        Self {
            full_name,
            sep_index,
        }
    }

    pub fn new_args(args: &[&str]) -> Self {
        let cli = TestCli::get();
        let user = TestCli::organisation();
        let full_name = format!("{}/test-dataset-{}", user, Uuid::new_v4());
        let sep_index = user.len();

        let output = cli.run(["create", "dataset", &full_name].iter().chain(args));
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

impl Drop for TestDataset {
    fn drop(&mut self) {
        let output = TestCli::get().run(&["delete", "dataset", self.identifier()]);
        assert!(output.is_empty());
    }
}

#[test]
fn test_test_dataset() {
    let cli = TestCli::get();
    let dataset = TestDataset::new();
    let identifier = dataset.identifier().to_owned();

    let output = cli.run(&["get", "datasets"]);
    assert!(output.contains(&identifier));

    drop(dataset);

    // RAII TestDataset; should automatically clean up the temporary dataset on drop.
    let output = cli.run(&["get", "datasets"]);
    assert!(!output.contains(&identifier));
}

#[test]
fn test_list_multiple_datasets() {
    let cli = TestCli::get();
    let dataset1 = TestDataset::new();
    let dataset2 = TestDataset::new();

    let output = cli.run(&["get", "datasets"]);
    assert!(output.contains(dataset1.identifier()));
    assert!(output.contains(dataset2.identifier()));

    let output = cli.run(&["get", "datasets", dataset1.identifier()]);
    assert!(output.contains(dataset1.identifier()));
    assert!(!output.contains(dataset2.identifier()));

    let output = cli.run(&["get", "datasets", dataset2.identifier()]);
    assert!(!output.contains(dataset1.identifier()));
    assert!(output.contains(dataset2.identifier()));
}

#[test]
fn test_create_dataset_custom() {
    let cli = TestCli::get();
    let source = TestSource::new();

    let dataset = TestDataset::new_args(&[
        "--title=some title",
        "--description=some description",
        &format!("--source={}", source.identifier()),
        "--has-sentiment=true",
    ]);

    let output = cli.run(&["get", "datasets", dataset.identifier(), "--output=json"]);
    let dataset_info: Dataset = serde_json::from_str(&output).unwrap();
    assert_eq!(&dataset_info.owner.0, dataset.owner());
    assert_eq!(&dataset_info.name.0, dataset.name());
    assert_eq!(dataset_info.title, "some title");
    assert_eq!(dataset_info.description, "some description");
    assert_eq!(dataset_info.source_ids.len(), 1);
    assert_eq!(dataset_info.has_sentiment, true);
}

#[test]
fn test_create_dataset_with_source() {
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset = TestDataset::new_args(&[&format!("--source={}", source.identifier())]);

    let output = cli.run(&["get", "datasets", "--output=json", dataset.identifier()]);
    let dataset_info: Dataset = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(&dataset_info.owner.0, dataset.owner());
    assert_eq!(&dataset_info.name.0, dataset.name());

    let source_output = cli.run(&[
        "get",
        "sources",
        "--output=json",
        &dataset_info.source_ids.first().unwrap().0,
    ]);
    let source_info: Source = serde_json::from_str(source_output.trim()).unwrap();
    assert_eq!(&source_info.owner.0, source.owner());
    assert_eq!(&source_info.name.0, source.name());
}

#[test]
fn test_create_dataset_requires_owner() {
    let cli = TestCli::get();

    let output = cli
        .command()
        .args(&["create", "dataset", "dataset-without-owner"])
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_create_dataset_model_family() {
    let cli = TestCli::get();
    let dataset = TestDataset::new_args(&["--model-family==german"]);

    let output = cli.run(&["get", "datasets", "--output=json", dataset.identifier()]);
    let dataset_info: Dataset = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(&dataset_info.owner.0, dataset.owner());
    assert_eq!(&dataset_info.name.0, dataset.name());
    assert_eq!(&dataset_info.model_family.0, "german");
}

#[test]
fn test_create_dataset_wrong_model_family() {
    let cli = TestCli::get();
    let output = cli
        .command()
        .args(&[
            "create",
            "dataset",
            "--model-family==non-existent-family",
            &format!(
                "{}/test-dataset-{}",
                TestCli::organisation(),
                Uuid::new_v4()
            ),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    dbg!(String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("API request failed with 400 Bad Request: 'non-existent-family' is not one of"))
}
