use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    resources::dataset::{DatasetAndStats, DatasetStats, StatisticsRequestParams},
    Client, DatasetIdentifier, SourceIdentifier,
};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetDatasetsArgs {
    #[structopt(name = "dataset")]
    /// If specified, only list this dataset (name or id)
    dataset: Option<DatasetIdentifier>,

    #[structopt(long = "stats")]
    /// Whether to include dataset statistics in response
    include_stats: bool,

    #[structopt(long = "source")]
    /// If specified, only list this datasets containing this source (name or id)
    source_identifier: Option<SourceIdentifier>,
}

pub fn get(client: &Client, args: &GetDatasetsArgs, printer: &Printer) -> Result<()> {
    let GetDatasetsArgs {
        dataset,
        include_stats,
        source_identifier,
    } = args;
    let mut datasets = if let Some(dataset) = dataset {
        vec![client
            .get_dataset(dataset.clone())
            .context("Operation to list datasets has failed.")?]
    } else {
        let mut datasets = client
            .get_datasets()
            .context("Operation to list datasets has failed.")?;
        datasets.sort_unstable_by(|lhs, rhs| {
            (&lhs.owner.0, &lhs.name.0).cmp(&(&rhs.owner.0, &rhs.name.0))
        });
        datasets
    };

    if let Some(source_id) = source_identifier {
        let source = client.get_source(source_id.clone())?;

        datasets.retain(|d| d.source_ids.contains(&source.id));
    }

    let mut dataset_stats = Vec::new();
    if *include_stats {
        datasets.iter().try_for_each(|dataset| -> Result<()> {
            info!("Getting statistics for dataset {}", dataset.full_name().0);
            let unfiltered_stats = client
                .get_dataset_statistics(
                    &dataset.full_name(),
                    &StatisticsRequestParams {
                        ..Default::default()
                    },
                )
                .context("Could not get statistics for dataset")?;

            let validation_response = client
                .get_latest_validation(&dataset.full_name())
                .context("Could not get validation for datase")?;

            let dataset_and_stats = DatasetAndStats {
                dataset: dataset.clone(),
                stats: DatasetStats {
                    num_reviewed: validation_response.validation.reviewed_size,
                    total_verbatims: unfiltered_stats.num_comments,
                    model_rating: validation_response.validation.model_rating,
                    latest_model_version: validation_response.validation.version,
                },
            };
            dataset_stats.push(dataset_and_stats);
            Ok(())
        })?;

        printer.print_resources(&dataset_stats)
    } else {
        printer.print_resources(&datasets)
    }
}
