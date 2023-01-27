use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{Client, DatasetIdentifier, SourceIdentifier, UpdateDataset};
use structopt::StructOpt;

/// Update a dataset.
#[derive(Clone, Debug, StructOpt)]
pub struct UpdateDatasetArgs {
    #[structopt(name = "dataset")]
    /// Name or id of the dataset to delete
    dataset: DatasetIdentifier,

    #[structopt(long = "title")]
    /// Set the title of the dataset
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the dataset
    description: Option<String>,

    #[structopt(short = "s", long = "source")]
    /// Names or ids of the sources in the dataset
    sources: Option<Vec<SourceIdentifier>>,
}

pub async fn update(client: &Client, args: UpdateDatasetArgs, printer: &Printer) -> Result<()> {
    let UpdateDatasetArgs {
        dataset,
        title,
        description,
        mut sources,
    } = args;

    let source_ids = match sources.take() {
        Some(sources) => Some(
            futures::future::try_join_all(sources.into_iter().map(|identifier| async {
                let source_id: Result<_> = Ok(client.get_source(identifier).await?.id);
                source_id
            }))
            .await?,
        ),
        None => None,
    };

    let dataset_full_name = match dataset {
        DatasetIdentifier::FullName(name) => name.to_owned(),
        dataset @ DatasetIdentifier::Id(_) => client
            .get_dataset(dataset.to_owned())
            .await
            .context("Fetching dataset id.")?
            .full_name(),
    };

    let dataset = client
        .update_dataset(
            &dataset_full_name,
            UpdateDataset {
                source_ids: source_ids.as_deref(),
                title: title.as_deref(),
                description: description.as_deref(),
            },
        )
        .await
        .context("Operation to update a dataset has failed.")?;
    info!(
        "Dataset `{}` [id: {}] updated successfully",
        dataset.full_name().0,
        dataset.id.0,
    );
    printer.print_resources(&[dataset])?;
    Ok(())
}
