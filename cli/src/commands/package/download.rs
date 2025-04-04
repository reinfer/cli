use std::{fs::remove_file, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use reinfer_client::{
    resources::dataset::{DatasetFlag, QueryRequestParams},
    Client, CommentFilter, Dataset, DatasetFullName, DatasetIdentifier, DatasetName, SourceId,
};
use structopt::StructOpt;

use crate::{
    commands::package::{PackageContentName, PackageWriter},
    DEFAULT_PROJECT_NAME,
};

#[derive(Debug, StructOpt)]
pub struct DownloadPackageArgs {
    /// The api name of the project
    project: DatasetName,

    /// Path to save the package to
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file: PathBuf,

    #[structopt(long)]
    /// Whether to overwrite existing package file
    overwrite: bool,

    #[structopt(long = "batch-size", default_value = "128")]
    /// Number of comments to batch in a single request.
    batch_size: usize,
}

fn get_project(project: &DatasetName, client: &Client) -> Result<Dataset> {
    let default_project_name = DEFAULT_PROJECT_NAME.clone();

    let dataset = client
        .get_dataset(DatasetIdentifier::FullName(
            project.clone().with_project(&default_project_name)?,
        ))
        .context(format!(
            "Could not get project with the name {0}",
            project.0
        ))?;

    if !dataset.has_flag(DatasetFlag::Ixp) {
        return Err(anyhow!(
            "Can only get packages for Unstructured and Complex Document projects"
        ));
    }

    Ok(dataset)
}

fn package_source(
    dataset: &DatasetFullName,
    source_id: &SourceId,
    batch_size: usize,
    package_writer: &mut PackageWriter,
    client: &Client,
) -> Result<()> {
    let mut query = QueryRequestParams {
        limit: Some(batch_size),
        filter: CommentFilter {
            sources: vec![source_id.clone()],
            ..Default::default()
        },
        ..Default::default()
    };

    let query_iter = client.get_dataset_query_iter(dataset, &mut query);

    package_writer.write_comment_query_iter_comments(
        PackageContentName::SourceComments(source_id.clone()),
        query_iter,
    )?;
    Ok(())
}

pub fn run(args: &DownloadPackageArgs, client: &Client) -> Result<()> {
    let DownloadPackageArgs {
        project,
        file,
        overwrite,
        batch_size,
    } = args;

    if *overwrite && file.is_file() {
        remove_file(file)?
    }

    let mut package_writer = PackageWriter::new(file.into())?;

    log::info!("Getting project...");
    let dataset = get_project(project, client)?;

    log::info!("Packaging dataset {0}", &dataset.id.0);
    package_writer.write_dataset(dataset.clone())?;

    for source_id in &dataset.source_ids {
        log::info!("Packaging source {0}...", source_id.0);
        package_source(
            &dataset.full_name(),
            source_id,
            *batch_size,
            &mut package_writer,
            client,
        )?;
    }

    package_writer.finish()?;
    log::info!("Package exported");
    Ok(())
}
