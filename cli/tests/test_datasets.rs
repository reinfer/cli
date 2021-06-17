use pretty_assertions::assert_eq;
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
fn test_create_update_dataset_custom() {
    let cli = TestCli::get();
    let dataset = TestDataset::new_args(&[
        "--title=some title",
        "--description=some description",
        "--has-sentiment=true",
    ]);

    /// A subset of source fields that we can easily check for equality accross
    #[derive(PartialEq, Eq, Debug)]
    struct DatasetInfo {
        owner: String,
        name: String,
        title: String,
        description: String,
        has_sentiment: bool,
        source_ids: Vec<String>,
    }

    impl From<Dataset> for DatasetInfo {
        fn from(dataset: Dataset) -> DatasetInfo {
            DatasetInfo {
                owner: dataset.owner.0,
                name: dataset.name.0,
                title: dataset.title,
                description: dataset.description,
                has_sentiment: dataset.has_sentiment,
                source_ids: dataset.source_ids.into_iter().map(|id| id.0).collect(),
            }
        }
    }

    let get_dataset_info = || -> DatasetInfo {
        let output = cli.run(&["get", "datasets", dataset.identifier(), "--output=json"]);
        serde_json::from_str::<Dataset>(&output).unwrap().into()
    };

    let mut expected_dataset_info = DatasetInfo {
        owner: dataset.owner().to_owned(),
        name: dataset.name().to_owned(),
        title: "some title".to_owned(),
        description: "some description".to_owned(),
        has_sentiment: true,
        source_ids: vec![],
    };
    assert_eq!(get_dataset_info(), expected_dataset_info);

    // Partial update
    cli.run(&[
        "update",
        "dataset",
        "--title=updated title",
        dataset.identifier(),
    ]);
    expected_dataset_info.title = "updated title".to_owned();
    assert_eq!(get_dataset_info(), expected_dataset_info);

    // Should be able to update all fields
    let test_source = TestSource::new();
    let source = test_source.get();
    cli.run(&[
        "update",
        "dataset",
        "--title=updated title",
        "--description=updated description",
        &format!("--source={}", source.id.0),
        dataset.identifier(),
    ]);

    expected_dataset_info.title = "updated title".to_owned();
    expected_dataset_info.description = "updated description".to_owned();
    expected_dataset_info.source_ids = vec![source.id.0];
    assert_eq!(get_dataset_info(), expected_dataset_info);

    // An empty update should be fine, including leaving source ids untouched
    cli.run(&["update", "dataset", dataset.identifier()]);
    assert_eq!(get_dataset_info(), expected_dataset_info);

    // Setting the sources flag with no ids should clear sources
    cli.run(&["update", "dataset", dataset.identifier(), "--source"]);
    expected_dataset_info.source_ids = vec![];
    assert_eq!(get_dataset_info(), expected_dataset_info);
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
    assert_eq!(dataset_info.source_ids.len(), 1);

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
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("API request failed with 400 Bad Request: 'non-existent-family' is not one of"))
}

#[test]
fn test_create_dataset_copy_annotations() {
    let cli = TestCli::get();
    let dataset1 = TestDataset::new();
    let dataset1_output = cli.run(&["get", "datasets", "--output=json", dataset1.identifier()]);
    let dataset1_info: Dataset = serde_json::from_str(dataset1_output.trim()).unwrap();

    let output = cli
        .command()
        .args(&[
            "create",
            "dataset",
            &format!("--copy-annotations-from={}", dataset1_info.id.0),
            &format!(
                "{}/test-dataset-{}",
                TestCli::organisation(),
                Uuid::new_v4()
            ),
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
}
