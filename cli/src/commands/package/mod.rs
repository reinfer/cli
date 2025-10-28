use anyhow::{Context, Result};
use download::DownloadPackageArgs;
use itertools::Itertools;
use openapi::{
    apis::configuration::Configuration,
    models::{AnnotatedComment, Bucket, Dataset, Email, EmailNew, Source},
};
// DatasetId is just String in OpenAPI models

use scoped_threadpool::Pool;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use upload::UploadPackageArgs;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};
mod download;
mod upload;

#[derive(Debug, StructOpt)]
pub enum PackageArgs {
    #[structopt(name = "download")]
    /// Download a package
    Download(DownloadPackageArgs),

    #[structopt(name = "upload")]
    /// Upload a package
    Upload(UploadPackageArgs),
}

pub struct EmailBatchKey(usize);

pub struct CommentBatchKey(usize);
#[derive(Clone)]
pub struct AttachmentKey(usize);

pub fn run(args: &PackageArgs, config: &Configuration, pool: &mut Pool) -> Result<()> {
    match args {
        PackageArgs::Download(args) => download::run(args, config, pool),
        PackageArgs::Upload(args) => upload::run(args, config, pool),
    }
}

pub enum PackageContentId<'a> {
    Dataset {
        dataset_id: &'a str,
    },
    Source {
        source_id: &'a str,
    },
    Bucket {
        bucket_id: &'a str,
    },
    CommentBatch {
        key: CommentBatchKey,
        source_id: &'a str,
    },
    EmailBatch {
        key: EmailBatchKey,
        bucket_id: &'a str,
    },
    Document {
        source_id: &'a str,
        comment_id: &'a str,
        key: &'a AttachmentKey,
        extension: Option<String>,
    },
}

static DATASET_POSTFIX_AND_EXTENSION: &str = "dataset.json";
static SOURCE_POSTFIX_AND_EXTENSION: &str = "source.json";
static COMMENTS_POSTFIX_AND_EXTENSION: &str = "comments.json";
static EMAILS_POSTFIX_AND_EXTENSION: &str = "emails.json";
static BUCKET_POSTFIX_AND_EXTENSION: &str = "buckets.json";
static DATASETS_FOLDER_NAME: &str = "datasets";
static SOURCES_FOLDER_NAME: &str = "sources";
static BUCKETS_FOLDER_NAME: &str = "buckets";
static EMAILS_FOLDER_NAME: &str = "emails";
static COMMENTS_FOLDER_NAME: &str = "comments";
static DOCUMENTS_FOLDER_NAME: &str = "documents";

impl PackageContentId<'_> {
    fn filename(&self) -> String {
        match self {
            PackageContentId::Bucket { bucket_id } => {
                format!(
                    "{BUCKETS_FOLDER_NAME}/{bucket_id}.{BUCKET_POSTFIX_AND_EXTENSION}"
                )
            }
            PackageContentId::Dataset { dataset_id } => {
                format!(
                    "{DATASETS_FOLDER_NAME}/{dataset_id}.{DATASET_POSTFIX_AND_EXTENSION}"
                )
            }
            PackageContentId::Source { source_id } => {
                format!(
                    "{SOURCES_FOLDER_NAME}/{source_id}.{SOURCE_POSTFIX_AND_EXTENSION}"
                )
            }
            PackageContentId::CommentBatch { key, source_id } => {
                format!(
                    "{COMMENTS_FOLDER_NAME}/{0}.{1}.{COMMENTS_POSTFIX_AND_EXTENSION}",
                    source_id, key.0
                )
            }
            PackageContentId::EmailBatch { key, bucket_id } => {
                format!(
                    "{EMAILS_FOLDER_NAME}/{0}.{1}.{EMAILS_POSTFIX_AND_EXTENSION}",
                    bucket_id, key.0
                )
            }
            PackageContentId::Document {
                source_id,
                comment_id,
                extension,
                key,
            } => {
                if let Some(extension) = extension {
                    format!(
                        "{DOCUMENTS_FOLDER_NAME}/{0}.{1}.{2}.document.{3}",
                        source_id, comment_id, key.0, extension
                    )
                } else {
                    format!("{0}.{1}.{2}.document", source_id, comment_id, key.0)
                }
            }
        }
    }

    fn friendly_name(&self) -> String {
        match self {
            PackageContentId::Dataset { dataset_id } => format!("dataset {dataset_id}"),
            PackageContentId::Source { source_id } => format!("source {source_id}"),
            PackageContentId::Bucket { bucket_id } => format!("bucket {bucket_id}"),
            PackageContentId::CommentBatch { key, source_id } => {
                format!("comment batch {0} for source {1}", key.0, source_id)
            }
            PackageContentId::EmailBatch { key, bucket_id } => {
                format!("email batch {0} for bucket {1}", key.0, bucket_id)
            }
            PackageContentId::Document {
                source_id,
                comment_id,
                key,
                extension,
            } => {
                let extension_part = if let Some(extension) = extension {
                    format!("{extension} ")
                } else {
                    String::new()
                };

                format!(
                    "{0}attachment for comment {1}, in source {2} with key {3}",
                    extension_part, comment_id, source_id, key.0
                )
            }
        }
    }
}

pub struct PackageWriter {
    writer: ZipWriter<File>,
}

pub struct Package {
    archive: ZipArchive<File>,
}

impl Package {
    fn new(path: &PathBuf) -> Result<Self> {
        let file = File::open(path)?;
        let archive = ZipArchive::new(file)?;

        Ok(Self { archive })
    }

    pub fn read_document(
        &mut self,
        source_id: &str,
        comment_id: &str,
        key: &AttachmentKey,
        extension: Option<String>,
    ) -> Result<Vec<u8>> {
        let content_id = PackageContentId::Document {
            source_id,
            comment_id,
            key,
            extension,
        };

        self.read_bytes(content_id)
    }

    pub fn read_bytes(&mut self, content_id: PackageContentId) -> Result<Vec<u8>> {
        let mut contents = Vec::new();
        let mut file = self.archive.by_name(&content_id.filename())?;

        file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    fn get_filenames_with_postfix_and_extension(&self, postfix: &str) -> Vec<String> {
        self.archive
            .file_names()
            .filter(|name| name.ends_with(postfix))
            .map(str::to_string)
            .collect()
    }

    pub fn get_source_by_id(&mut self, source_id: &str) -> Result<Source> {
        self.read_json_content_by_id(PackageContentId::Source { source_id })
    }

    pub fn get_bucket_by_id(&mut self, bucket_id: &str) -> Result<Bucket> {
        self.read_json_content_by_id(PackageContentId::Bucket { bucket_id })
    }
    pub fn get_email_batch(
        &mut self,
        bucket_id: &str,
        key: EmailBatchKey,
    ) -> Result<Vec<EmailNew>> {
        let content_id = PackageContentId::EmailBatch { key, bucket_id };

        self.read_jsonl_content_by_id(content_id)
    }

    pub fn get_email_batch_count_for_bucket(&mut self, bucket_id: &str) -> usize {
        self.get_filenames_with_postfix_and_extension(EMAILS_POSTFIX_AND_EXTENSION)
            .iter()
            .filter(|filename| {
                let path = Path::new(filename);
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with(bucket_id))
            })
            .count()
    }
    pub fn get_comment_batch(
        &mut self,
        source_id: &str,
        key: CommentBatchKey,
    ) -> Result<Vec<AnnotatedComment>> {
        let content_id = PackageContentId::CommentBatch { key, source_id };

        self.read_jsonl_content_by_id(content_id)
    }

    pub fn get_comment_batch_count_for_source(&mut self, source_id: &str) -> usize {
        self.get_filenames_with_postfix_and_extension(COMMENTS_POSTFIX_AND_EXTENSION)
            .iter()
            .filter(|filename| {
                let path = Path::new(filename);
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with(source_id))
            })
            .count()
    }

    pub fn get_document_count(&mut self) -> usize {
        self.archive
            .file_names()
            .filter(|name| {
                let path = Path::new(name);
                path.parent()
                    .is_some_and(|folder| folder.to_string_lossy() == DOCUMENTS_FOLDER_NAME)
            })
            .count()
    }

    pub fn get_comment_batch_count(&mut self) -> usize {
        self.archive
            .file_names()
            .filter(|name| {
                let path = Path::new(name);
                path.parent()
                    .is_some_and(|folder| folder.to_string_lossy() == COMMENTS_FOLDER_NAME)
            })
            .count()
    }

    pub fn get_emails_batch_count(&mut self) -> usize {
        self.archive
            .file_names()
            .filter(|name| {
                let path = Path::new(name);
                path.parent()
                    .is_some_and(|folder| folder.to_string_lossy() == EMAILS_FOLDER_NAME)
            })
            .count()
    }

    pub fn datasets(&mut self) -> Result<Vec<Dataset>> {
        let dataset_filenames =
            self.get_filenames_with_postfix_and_extension(DATASET_POSTFIX_AND_EXTENSION);

        dataset_filenames
            .iter()
            .map(|filename| self.read_json_content_by_name(filename))
            .try_collect()
    }

    fn read_string_content_by_name(&mut self, filename: &str) -> Result<String> {
        let mut file = self.archive.by_name(filename)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    fn read_json_content_by_name<T>(&mut self, filename: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let string = self.read_string_content_by_name(filename)?;

        Ok(serde_json::from_str(&string)?)
    }

    fn read_jsonl_content_by_id<T>(&mut self, content: PackageContentId) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let string = self
            .read_string_content_by_name(&content.filename())
            .context(format!(
                "Package does not contain a valid jsonl record for {}",
                content.friendly_name()
            ))?;

        string
            .lines()
            .map(|line| -> Result<T> {
                serde_json::from_str::<T>(line).map_err(anyhow::Error::msg)
            })
            .try_collect()
    }

    fn read_json_content_by_id<T>(&mut self, content: PackageContentId) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.read_json_content_by_name(&content.filename())
            .context(format!(
                "Package does not contain a valid json record for {}",
                content.friendly_name()
            ))
    }
}

impl PackageWriter {
    pub fn new(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        let writer = ZipWriter::new(file);
        Ok(Self { writer })
    }

    pub fn write_dataset(&mut self, dataset: &Dataset) -> Result<()> {
        let dataset_id = &dataset.id;

        self.write_json(PackageContentId::Dataset { dataset_id }, dataset)
    }

    pub fn write_source(&mut self, source: &Source) -> Result<()> {
        let source_id = &source.id;
        self.write_json(PackageContentId::Source { source_id }, source)
    }

    pub fn write_bucket(&mut self, bucket: &Bucket) -> Result<()> {
        let bucket_id = &bucket.id;
        self.write_json(PackageContentId::Bucket { bucket_id }, bucket)
    }

    pub fn write_email_batch(
        &mut self,
        bucket_id: &str,
        key: EmailBatchKey,
        emails: &[Email],
    ) -> Result<()> {
        self.write_jsonl(PackageContentId::EmailBatch { key, bucket_id }, emails)
    }

    pub fn write_comment_batch(
        &mut self,
        source_id: &str,
        key: CommentBatchKey,
        comments: &[AnnotatedComment],
    ) -> Result<()> {
        self.write_jsonl(PackageContentId::CommentBatch { key, source_id }, comments)
    }

    pub fn write_bytes(&mut self, content_id: PackageContentId, content: &[u8]) -> Result<()> {
        self.writer
            .start_file_from_path(content_id.filename(), SimpleFileOptions::default())?;
        self.writer.write_all(content).map_err(anyhow::Error::msg)
    }

    fn write_jsonl<T>(&mut self, content_id: PackageContentId, content: &[T]) -> Result<()>
    where
        T: Serialize,
    {
        self.writer
            .start_file_from_path(content_id.filename(), SimpleFileOptions::default())?;

        content.iter().try_for_each(|item| -> Result<()> {
            let json = serde_json::to_string(item)?;

            self.writer
                .write_all(format!("{json}\n").as_bytes())
                .map_err(anyhow::Error::msg)
        })
    }

    fn write_json<T>(&mut self, content_id: PackageContentId, content: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.writer
            .start_file_from_path(content_id.filename(), SimpleFileOptions::default())?;
        let json_content = serde_json::to_string_pretty(content)?;

        self.writer.write_all(json_content.as_bytes())?;
        Ok(())
    }

    fn finish(self) -> Result<()> {
        self.writer.finish()?;
        Ok(())
    }
}
