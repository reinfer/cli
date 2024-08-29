use std::sync::mpsc::channel;

use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    resources::dataset::{DatasetAndStats, DatasetStats, StatisticsRequestParams},
    Client, DatasetIdentifier, SourceIdentifier,
};
use scoped_threadpool::Pool;
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

pub fn get(
    client: &Client,
    args: &GetDatasetsArgs,
    printer: &Printer,
    pool: &mut Pool,
) -> Result<()> {
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

    let (sender, receiver) = channel();

    if *include_stats {
        pool.scoped(|scope| {
            datasets.iter().for_each(|dataset| {
                let get_stats = || -> Result<DatasetAndStats> {
                    info!("Getting statistics for dataset {}", dataset.full_name().0);
                    let unfiltered_stats = client
                        .get_dataset_statistics(
                            &dataset.full_name(),
                            &StatisticsRequestParams {
                                ..Default::default()
                            },
                        )
                        .context("Could not get statistics for dataset")?;

                    let validation_response = client.get_latest_validation(&dataset.full_name());

                    Ok(DatasetAndStats {
                        dataset: dataset.clone(),
                        stats: DatasetStats {
                            total_verbatims: unfiltered_stats.num_comments,
                            validation: validation_response.ok(),
                        },
                    })
                };

                let sender = sender.clone();
                scope.execute(move || {
                    sender.send(get_stats()).expect("Could not send error");
                });
            });
        });

        drop(sender);
        let mut dataset_stats = Vec::new();
        let results: Vec<Result<DatasetAndStats>> = receiver.iter().collect();

        for result in results {
            dataset_stats.push(result?);
        }

        printer.print_resources(&dataset_stats)
    } else {
        printer.print_resources(&datasets)
    }
}
