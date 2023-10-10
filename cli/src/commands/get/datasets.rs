use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    resources::dataset::{DatasetAndStats, DatasetStats, StatisticsRequestParams},
    Client, CommentFilter, DatasetIdentifier,
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
}

pub fn get(client: &Client, args: &GetDatasetsArgs, printer: &Printer) -> Result<()> {
    let GetDatasetsArgs {
        dataset,
        include_stats,
    } = args;
    let datasets = if let Some(dataset) = dataset {
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

    let mut dataset_stats = Vec::new();
    if *include_stats {
        datasets.iter().try_for_each(|dataset| -> Result<()> {
            info!("Getting statistics for dataset {}", dataset.full_name().0);
            let stats = client
                .get_dataset_statistics(
                    &dataset.full_name(),
                    &StatisticsRequestParams {
                        comment_filter: CommentFilter {
                            reviewed:Some(reinfer_client::resources::comment::ReviewedFilterEnum::OnlyReviewed),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )
                .context("Could not get statistics for dataset")?;

            let dataset_and_stats = DatasetAndStats {
                dataset: dataset.clone(),
                stats: DatasetStats {
                    num_reviewed: stats.num_comments
                }
            };
            dataset_stats.push(dataset_and_stats);
            Ok(())
        })?;
        printer.print_resources(&dataset_stats)
    } else {
        printer.print_resources(&datasets)
    }
}
