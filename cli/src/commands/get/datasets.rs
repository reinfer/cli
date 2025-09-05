use std::sync::mpsc::channel;

use anyhow::{Context, Result};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        datasets_api::{get_all_datasets, get_dataset, get_dataset_statistics},
        models_api::get_validation,
        sources_api::{get_source, get_source_by_id},
    },
    models::{Dataset, GetDatasetStatisticsRequest, Statistics, GetValidationResponse},
};
use scoped_threadpool::Pool;
use structopt::StructOpt;

use crate::printer::Printer;
use crate::utils::resource_identifier::{DatasetIdentifier, SourceIdentifier};

// Simple wrapper for dataset with stats
#[derive(Debug, Clone)]
pub struct DatasetWithStats {
    pub dataset: Dataset,
    pub stats: Statistics,
    pub validation: Option<GetValidationResponse>,
}

#[derive(Debug, StructOpt)]
pub struct GetDatasetsArgs {
    #[structopt(name = "dataset")]
    /// If specified, only list this dataset (name or id)
    dataset: Option<DatasetIdentifier>,

    #[structopt(long = "stats")]
    /// Whether to include dataset statistics in response
    include_stats: bool,

    #[structopt(long = "source")]
    /// If specified, only list the datasets containing this source (name or id)
    source_identifier: Option<SourceIdentifier>,
}

pub fn get(
    config: &Configuration,
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
        let dataset_response = match &dataset {
            DatasetIdentifier::Id(id) => {
                // For now, we'll need to get dataset by ID - this might need a different API call
                return Err(anyhow::anyhow!("Dataset lookup by ID is not supported. Please use the full name format 'owner/dataset'"));
            }
            DatasetIdentifier::FullName(full_name) => {
                get_dataset(config, full_name.owner(), full_name.name())
                    .context("Operation to list datasets has failed.")?
            }
        };
        vec![dataset_response.dataset]
    } else {
        let response = get_all_datasets(config)
            .context("Operation to list datasets has failed.")?;
        let mut datasets = response.datasets;
        datasets.sort_unstable_by(|lhs, rhs| {
            (&lhs.owner, &lhs.name).cmp(&(&rhs.owner, &rhs.name))
        });
        datasets
    };

    if let Some(source_id) = source_identifier {
        let source = match &source_id {
            SourceIdentifier::Id(id) => {
                get_source_by_id(config, id)
                    .context("Failed to get source")?
                    .source
            }
            SourceIdentifier::FullName(full_name) => {
                get_source(config, full_name.owner(), full_name.name())
                    .context("Failed to get source")?
                    .source
            }
        };

        datasets.retain(|d| d.source_ids.contains(&source.id));
    }

    if *include_stats {
        let (sender, receiver) = channel();

        pool.scoped(|scope| {
            datasets.iter().for_each(|dataset| {
                let get_stats = || -> Result<DatasetWithStats> {
                    info!("Getting statistics for dataset {}/{}", dataset.owner, dataset.name);
                    
                    let request = GetDatasetStatisticsRequest {
                        comment_filter: None,
                        label_filter: None,
                        by_labels: None,
                        by_label_properties: vec![], // Empty for basic stats
                        time_resolution: None,
                        string_user_property_counts: None,
                        email_property_counts: None,
                        source_counts: None,
                        label_timeseries: None,
                        label_property_timeseries: None,
                        thread_histogram: None,
                        nps_property: None,
                        attribute_filters: None,
                        thread_mode: None,
                        timezone: None,
                    };

                    let response = get_dataset_statistics(config, &dataset.owner, &dataset.name, request)
                        .context("Could not get statistics for dataset")?;

                    // Get validation (equivalent to get_latest_validation)
                    let validation_response = get_validation(config, &dataset.owner, &dataset.name, "latest", None).ok();

                    Ok(DatasetWithStats {
                        dataset: dataset.clone(),
                        stats: response.statistics,
                        validation: validation_response,
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
        let results: Vec<Result<DatasetWithStats>> = receiver.iter().collect();

        for result in results {
            dataset_stats.push(result?);
        }

        printer.print_resources(&dataset_stats)
    } else {
        printer.print_resources(&datasets)
    }
}
