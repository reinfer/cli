use std::collections::HashMap;

use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    resources::dataset::StatisticsRequestParams, Client, CommentFilter, DatasetIdentifier,
};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetDatasetsArgs {
    #[structopt(name = "dataset")]
    /// If specified, only list this dataset (name or id)
    dataset: Option<DatasetIdentifier>,

    #[structopt(long = "stats")]
    /// Whether to include source statistics in response
    include_stats: bool,
}

pub fn get(client: &Client, args: &GetDatasetsArgs, printer: &Printer) -> Result<()> {
    let GetDatasetsArgs {
        dataset,
        include_stats,
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

    let mut dataset_stats: HashMap<_, _> = HashMap::new();
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

            dataset_stats.insert(dataset.id.clone(), stats);
            Ok(())
        })?;

        datasets.iter_mut().for_each(|dataset| {
            dataset.num_reviewed = if let Some(stats) = dataset_stats.get(&dataset.id).cloned() {
                Some(*stats.num_comments)
            } else {
                None
            }
        });
    }

    printer.print_resources(&datasets)
}
