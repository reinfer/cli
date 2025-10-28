use std::sync::mpsc::channel;

use anyhow::{Context, Result};
use log::info;
use scoped_threadpool::Pool;
use structopt::StructOpt;

use openapi::{
    apis::{
        configuration::Configuration,
        datasets_api::{get_all_datasets, get_dataset_statistics},
        models_api::get_validation,
    },
    models::{Dataset, GetDatasetStatisticsRequest, GetValidationResponse, Statistics},
};

use crate::{
    printer::Printer,
    utils::{resolve_dataset, resolve_source, DatasetIdentifier, SourceIdentifier},
};

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

/// Retrieve datasets with optional statistics and source filtering
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

    let mut datasets = if let Some(dataset_identifier) = dataset {
        vec![resolve_dataset(config, dataset_identifier)?]
    } else {
        get_all_datasets_sorted(config)?
    };

    if let Some(source_identifier) = source_identifier {
        filter_datasets_by_source(config, &mut datasets, source_identifier)?;
    }

    let printable_datasets = if *include_stats {
        collect_datasets_with_stats(config, datasets, pool)?
    } else {
        create_printable_datasets_without_stats(datasets)
    };

    printer.print_resources(&printable_datasets)
}

/// Retrieve all datasets, sorted by owner and name
fn get_all_datasets_sorted(config: &Configuration) -> Result<Vec<Dataset>> {
    let response = get_all_datasets(config).context("Failed to list datasets")?;

    let mut datasets = response.datasets;
    datasets.sort_unstable_by(|lhs, rhs| (&lhs.owner, &lhs.name).cmp(&(&rhs.owner, &rhs.name)));

    Ok(datasets)
}

/// Filter datasets to only include those containing the specified source
fn filter_datasets_by_source(
    config: &Configuration,
    datasets: &mut Vec<Dataset>,
    source_identifier: &SourceIdentifier,
) -> Result<()> {
    let source = resolve_source(config, source_identifier)?;
    datasets.retain(|dataset| dataset.source_ids.contains(&source.id));
    Ok(())
}

/// Collect datasets with statistics using parallel processing
fn collect_datasets_with_stats(
    config: &Configuration,
    datasets: Vec<Dataset>,
    pool: &mut Pool,
) -> Result<Vec<crate::printer::PrintableDatasetWithStats>> {
    let (sender, receiver) = channel();

    pool.scoped(|scope| {
        for dataset in &datasets {
            let sender = sender.clone();
            let dataset = dataset.clone();

            scope.execute(move || {
                let result = collect_single_dataset_stats(config, &dataset);
                sender.send(result).expect("Failed to send result");
            });
        }
    });

    drop(sender);
    let results: Vec<Result<DatasetWithStats>> = receiver.iter().collect();

    let mut dataset_stats = Vec::new();
    for result in results {
        dataset_stats.push(result?);
    }

    Ok(dataset_stats
        .into_iter()
        .map(|ds| crate::printer::PrintableDatasetWithStats {
            dataset: ds.dataset,
            stats: Some(ds.stats),
            validation: ds.validation,
        })
        .collect())
}

/// Collect statistics and validation data for a single dataset
fn collect_single_dataset_stats(
    config: &Configuration,
    dataset: &Dataset,
) -> Result<DatasetWithStats> {
    info!(
        "Getting statistics for dataset {}/{}",
        dataset.owner, dataset.name
    );

    let request = create_basic_statistics_request();
    let response = get_dataset_statistics(config, &dataset.owner, &dataset.name, request)
        .with_context(|| {
            format!(
                "Failed to get statistics for dataset {}/{}",
                dataset.owner, dataset.name
            )
        })?;

    // Get validation data to show validation status
    let validation_response =
        get_validation(config, &dataset.owner, &dataset.name, "latest", None).ok();

    Ok(DatasetWithStats {
        dataset: dataset.clone(),
        stats: *response.statistics,
        validation: validation_response,
    })
}

/// Create a basic dataset statistics request
fn create_basic_statistics_request() -> GetDatasetStatisticsRequest {
    GetDatasetStatisticsRequest {
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
    }
}

/// Create printable datasets without statistics
fn create_printable_datasets_without_stats(
    datasets: Vec<Dataset>,
) -> Vec<crate::printer::PrintableDatasetWithStats> {
    datasets
        .into_iter()
        .map(|dataset| crate::printer::PrintableDatasetWithStats {
            dataset,
            stats: None,
            validation: None,
        })
        .collect()
}
