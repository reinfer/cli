use anyhow::{Context, Result};
use download::DownloadPackageArgs;
use reinfer_client::{Client, Dataset, DatasetQueryIter, SourceId};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
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

pub fn run(args: &PackageArgs, client: Client) -> Result<()> {
    match args {
        PackageArgs::Download(args) => download::run(args, &client),
        PackageArgs::Upload(args) => upload::run(args, &client),
    }
}

pub enum PackageContentName {
    Dataset,
    SourceComments(SourceId),
}

impl PackageContentName {
    fn filename(&self) -> String {
        match self {
            PackageContentName::Dataset => "dataset.json".to_string(),
            PackageContentName::SourceComments(source_id) => {
                format!("{0}.comments.jsonl", source_id.0)
            }
        }
    }

    fn friendly_name(&self) -> String {
        match self {
            PackageContentName::Dataset => "dataset".to_string(),
            PackageContentName::SourceComments(source_id) => {
                format!("source `{0}` comments", source_id.0)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum PackageContent {
    Dataset(Dataset),
}

impl PackageContent {
    fn path(&self) -> PathBuf {
        match self {
            Self::Dataset(_) => Path::new(&PackageContentName::Dataset.filename()).to_path_buf(),
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
    fn new(path: PathBuf) -> Result<Self> {
        let file = File::open(path)?;
        let archive = ZipArchive::new(file)?;

        Ok(Self { archive })
    }

    pub fn read_dataset(&mut self) -> Result<Dataset> {
        self.read_content(PackageContentName::Dataset)
    }

    fn read_content<T>(&mut self, content: PackageContentName) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut entry = self.archive.by_name(&content.filename()).context(format!(
            "Package does not contain a valid record for {}",
            content.friendly_name()
        ))?;

        let mut buf = String::new();
        entry.read_to_string(&mut buf)?;
        Ok(serde_json::from_str(&buf)?)
    }
}

impl PackageWriter {
    pub fn new(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        let writer = ZipWriter::new(file);
        Ok(Self { writer })
    }

    pub fn write_dataset(&mut self, dataset: Dataset) -> Result<()> {
        self.write_content(PackageContent::Dataset(dataset))
    }

    pub fn write_comment_query_iter_comments(
        &mut self,
        content_name: PackageContentName,
        mut query_iter: DatasetQueryIter<'_>,
    ) -> Result<()> {
        self.writer
            .start_file_from_path(content_name.filename(), SimpleFileOptions::default())?;

        query_iter.try_for_each(|result| -> Result<()> {
            let comments = result?;

            comments
                .iter()
                .try_for_each(|comment| self.write_json(comment))
        })
    }

    fn write_json<T>(&mut self, content: T) -> Result<()>
    where
        T: Serialize,
    {
        let json_content = json!(content).to_string();

        self.writer
            .write_all(format!("{json_content}\n").as_bytes())?;

        Ok(())
    }

    fn write_content(&mut self, content: PackageContent) -> Result<()> {
        self.writer
            .start_file_from_path(content.path(), SimpleFileOptions::default())?;
        self.write_json(content)
    }

    fn finish(self) -> Result<()> {
        self.writer.finish()?;
        Ok(())
    }
}
