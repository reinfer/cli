use anyhow::{Context, Result};
use reinfer_client::{Client, DatasetIdentifier};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetDatasetsArgs {
    #[structopt(name = "dataset")]
    /// If specified, only list this dataset (name or id)
    dataset: Option<DatasetIdentifier>,
}

pub async fn get(client: &Client, args: &GetDatasetsArgs, printer: &Printer) -> Result<()> {
    let GetDatasetsArgs { dataset } = args;
    let datasets = if let Some(dataset) = dataset {
        vec![client
            .get_dataset(dataset.clone())
            .await
            .context("Operation to list datasets has failed.")?]
    } else {
        let mut datasets = client
            .get_datasets()
            .await
            .context("Operation to list datasets has failed.")?;
        datasets.sort_unstable_by(|lhs, rhs| {
            (&lhs.owner.0, &lhs.name.0).cmp(&(&rhs.owner.0, &rhs.name.0))
        });
        datasets
    };
    printer.print_resources(&datasets)
}
