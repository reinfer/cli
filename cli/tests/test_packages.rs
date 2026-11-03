use crate::{test_comments::get_comments_with_delay, TestCli, TestDataset, TestSource};
use pretty_assertions::assert_eq;
use reinfer_client::{AnnotatedComment, Bucket, Dataset, NewAnnotatedComment, Source, SourceId};
use serde::de::DeserializeOwned;
use std::{
    fs,
    io::{Cursor, Read},
    path::PathBuf,
};
use uuid::Uuid;
use zip::ZipArchive;

const COMMENTS: &str = include_str!("./samples/labelling.jsonl");

struct Fixture {
    dataset: Option<TestDataset>,
    sources: Vec<TestSource>,
    buckets: Vec<String>,
}

impl Fixture {
    fn new(comments: &str) -> Self {
        let cli = TestCli::get();
        let mut sources = Vec::new();
        let mut buckets = Vec::new();
        for _ in 0..2 {
            let bucket = format!("{}/test-package-{}", TestCli::project(), Uuid::new_v4());
            cli.run(["create", "bucket", &bucket]);
            sources.push(TestSource::new_args(&["--bucket", &bucket]));
            buckets.push(bucket);
        }
        let dataset = TestDataset::new_args(&[
            &format!("--source={}", sources[0].identifier()),
            &format!("--source={}", sources[1].identifier()),
            "--label-defs",
            r#"[{"name":"A"},{"name":"B"},{"name":"C"}]"#,
        ]);
        for source in &sources {
            cli.run_with_stdin(
                [
                    "create",
                    "comments",
                    "--allow-duplicates",
                    "--yes",
                    "--source",
                    source.identifier(),
                    "--dataset",
                    dataset.identifier(),
                ],
                comments.as_bytes(),
            );
        }

        Self {
            dataset: Some(dataset),
            sources,
            buckets,
        }
    }

    fn dataset(&self) -> &TestDataset {
        self.dataset.as_ref().unwrap()
    }

    fn dataset_info(&self) -> Dataset {
        let output = TestCli::get().run([
            "--output=json",
            "get",
            "datasets",
            self.dataset().identifier(),
        ]);
        serde_json::from_str(&output).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Buckets can only be deleted after their sources and dataset.
        self.dataset.take();
        self.sources.clear();
        for bucket in &self.buckets {
            let _ = TestCli::get().run_and_result(["delete", "bucket", bucket]);
        }
    }
}

fn package_path() -> PathBuf {
    std::env::temp_dir().join(format!("re-package-test-{}.zip", Uuid::new_v4()))
}

fn download_package(dataset: &TestDataset, flags: &[&str]) -> ZipArchive<Cursor<Vec<u8>>> {
    let path = package_path();
    let args = [
        "package",
        "download",
        dataset.identifier(),
        "--file",
        path.to_str().unwrap(),
    ];
    TestCli::get().run(args.iter().chain(flags));
    let bytes = fs::read(&path).unwrap();
    fs::remove_file(path).unwrap();
    ZipArchive::new(Cursor::new(bytes)).unwrap()
}

fn read_json<T: DeserializeOwned>(archive: &mut ZipArchive<Cursor<Vec<u8>>>, name: &str) -> T {
    serde_json::from_reader(archive.by_name(name).unwrap()).unwrap()
}

fn assert_package(
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
    original: &Dataset,
    sources: &[Source],
    expected_comments: &[AnnotatedComment],
) {
    let mut metadata: serde_json::Map<String, serde_json::Value> =
        read_json(archive, &format!("datasets/{}.dataset.json", original.id.0));
    let mut actual_ids: Vec<SourceId> =
        serde_json::from_value(metadata.remove("source_ids").unwrap()).unwrap();
    actual_ids.sort_by(|left, right| left.0.cmp(&right.0));
    let mut expected_ids: Vec<_> = sources.iter().map(|source| source.id.clone()).collect();
    expected_ids.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(actual_ids, expected_ids);
    let mut expected_metadata = serde_json::to_value(original).unwrap();
    expected_metadata
        .as_object_mut()
        .unwrap()
        .remove("source_ids");
    assert_eq!(serde_json::Value::Object(metadata), expected_metadata);

    let filenames: Vec<_> = archive.file_names().map(str::to_owned).collect();
    assert_eq!(
        filenames
            .iter()
            .filter(|name| name.starts_with("sources/"))
            .count(),
        sources.len()
    );
    assert_eq!(
        filenames
            .iter()
            .filter(|name| name.starts_with("buckets/"))
            .count(),
        sources.len()
    );
    assert!(filenames.iter().all(|name| !name.starts_with("emails/")));

    for source in sources {
        let packaged_source: Source =
            read_json(archive, &format!("sources/{}.source.json", source.id.0));
        assert_eq!(packaged_source, *source);
        let bucket_id = source.bucket_id.as_ref().unwrap();
        let bucket: Bucket = read_json(archive, &format!("buckets/{}.buckets.json", bucket_id.0));
        assert_eq!(bucket.id, *bucket_id);
    }

    let mut comments = Vec::new();
    for name in filenames
        .iter()
        .filter(|name| name.starts_with("comments/"))
    {
        assert!(sources
            .iter()
            .any(|source| name.starts_with(&format!("comments/{}.", source.id.0))));
        let mut contents = String::new();
        archive
            .by_name(name)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        comments.extend(
            contents
                .lines()
                .map(|line| serde_json::from_str::<AnnotatedComment>(line).unwrap()),
        );
    }
    assert_eq!(comments.len(), expected_comments.len());
    for expected in expected_comments {
        let exported = comments
            .iter()
            .find(|comment| comment.comment.uid == expected.comment.uid)
            .unwrap();
        assert_eq!(exported.comment, expected.comment);
        assert_eq!(exported.labelling, expected.labelling);
    }
}

#[test]
fn test_package_download_reviewed_source_by_name_and_id() {
    let fixture = Fixture::new(COMMENTS);
    let dataset = fixture.dataset_info();
    let sources: Vec<_> = fixture.sources.iter().map(TestSource::get).collect();
    let reviewed_count = COMMENTS
        .lines()
        .map(|line| serde_json::from_str::<NewAnnotatedComment>(line).unwrap())
        .filter(NewAnnotatedComment::has_annotations)
        .count();
    let reviewed: Vec<AnnotatedComment> = sources
        .iter()
        .flat_map(|source| {
            let output = TestCli::get().run([
                "get",
                "comments",
                &source.full_name().0,
                "--dataset",
                fixture.dataset().identifier(),
                "--reviewed-only",
                "true",
            ]);
            let comments: Vec<AnnotatedComment> = output
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            assert_eq!(comments.len(), reviewed_count);
            comments
        })
        .collect();

    for identifier in [sources[0].full_name().0, sources[0].id.0.clone()] {
        let mut archive = download_package(
            fixture.dataset(),
            &["--source", &identifier, "--reviewed-only", "--skip-emails"],
        );
        assert_package(
            &mut archive,
            &dataset,
            &sources[..1],
            &reviewed[..reviewed_count],
        );
    }

    let mut archive = download_package(fixture.dataset(), &["--reviewed-only", "--skip-emails"]);
    assert_package(&mut archive, &dataset, &sources, &reviewed);
}

#[test]
fn test_package_download_source_without_reviewed_only() {
    let fixture = Fixture::new(COMMENTS);
    let dataset = fixture.dataset_info();
    let selected = fixture.sources[0].get();
    let output = get_comments_with_delay(
        TestCli::get(),
        &[
            "get",
            "comments",
            "--dataset",
            fixture.dataset().identifier(),
            fixture.sources[0].identifier(),
        ],
        COMMENTS.lines().count(),
    );
    let comments: Vec<AnnotatedComment> = output
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    let mut archive = download_package(
        fixture.dataset(),
        &["--source", &selected.id.0, "--skip-emails"],
    );
    assert_package(&mut archive, &dataset, &[selected], &comments);
}

#[test]
fn test_package_download_source_with_no_reviewed_comments() {
    let fixture = Fixture::new(include_str!("./samples/basic.jsonl"));
    let dataset = fixture.dataset_info();
    let selected = fixture.sources[0].get();
    let mut archive = download_package(
        fixture.dataset(),
        &[
            "--source",
            &selected.id.0,
            "--reviewed-only",
            "--skip-emails",
        ],
    );
    assert_package(&mut archive, &dataset, &[selected], &[]);
}

#[test]
fn test_package_download_invalid_source_preserves_existing_output() {
    let cli = TestCli::get();
    let source = TestSource::new();
    let dataset = TestDataset::new();
    let unknown = format!("{}/missing-source-{}", TestCli::project(), Uuid::new_v4());
    let path = package_path();

    for (identifier, message) in [
        (source.identifier(), "is not linked to dataset"),
        (unknown.as_str(), "Failed to get source"),
    ] {
        fs::write(&path, b"existing package").unwrap();
        let output = cli.run_and_error([
            "package",
            "download",
            dataset.identifier(),
            "--source",
            identifier,
            "--skip-emails",
            "--file",
            path.to_str().unwrap(),
            "--overwrite",
        ]);
        assert!(output.contains(message), "{output}");
        assert_eq!(fs::read(&path).unwrap(), b"existing package");
    }

    let output = cli.run_and_error([
        "package",
        "download",
        dataset.identifier(),
        "--source",
        source.identifier(),
        "--reviewed-only",
        "--file",
        path.to_str().unwrap(),
        "--overwrite",
    ]);
    assert!(output.contains("You must skip raw emails"), "{output}");
    assert_eq!(fs::read(&path).unwrap(), b"existing package");
    fs::remove_file(path).unwrap();
}
