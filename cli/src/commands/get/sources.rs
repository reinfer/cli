use crate::{
    printer::{PrintableSource, Printer},
    utils::{resolve_source, SourceIdentifier},
};
use anyhow::{Context, Result};
use log::info;
use std::collections::HashMap;
use std::str::FromStr;
use structopt::StructOpt;

use openapi::{
    apis::{
        buckets_api::get_all_buckets,
        configuration::Configuration,
        sources_api::{get_all_sources, get_source_statistics},
    },
    models,
};

#[derive(Debug, StructOpt)]
pub struct GetSourcesArgs {
    /// If specified, only list this source (name or id)
    #[structopt(name = "source", parse(try_from_str = SourceIdentifier::from_str))]
    source: Option<SourceIdentifier>,
    /// Whether to include source statistics in response
    #[structopt(long = "stats")]
    include_stats: bool,
}

/// Retrieve sources with optional statistics
pub fn get(config: &Configuration, args: &GetSourcesArgs, printer: &Printer) -> Result<()> {
    let GetSourcesArgs {
        source,
        include_stats,
    } = args;

    let sources = if let Some(source_identifier) = source {
        vec![resolve_source(config, source_identifier)?]
    } else {
        get_all_sources_sorted(config)?
    };

    let buckets_by_id = get_buckets_map(config)?;
    let source_stats = if *include_stats {
        collect_source_statistics(config, &sources)?
    } else {
        HashMap::new()
    };

    let printable_sources = create_printable_sources(sources, buckets_by_id, source_stats);
    printer.print_resources(&printable_sources)
}

/// Retrieve all sources, sorted by owner and name  
fn get_all_sources_sorted(config: &Configuration) -> Result<Vec<models::Source>> {
    let mut sources = get_all_sources(config)
        .context("Failed to list sources")?
        .sources;

    sources.sort_unstable_by(|a, b| (&a.owner, &a.name).cmp(&(&b.owner, &b.name)));
    Ok(sources)
}

/// Get a map of bucket ID to bucket for lookups
fn get_buckets_map(config: &Configuration) -> Result<HashMap<String, models::Bucket>> {
    let buckets = get_all_buckets(config)
        .context("Failed to list buckets")?
        .buckets;

    Ok(buckets
        .into_iter()
        .map(|bucket| (bucket.id.clone(), bucket))
        .collect())
}

/// Collect statistics for all provided sources
fn collect_source_statistics(
    config: &Configuration,
    sources: &[models::Source],
) -> Result<HashMap<String, models::SourceStatistics>> {
    let mut source_stats = HashMap::new();

    for source in sources {
        info!(
            "Getting statistics for source {}/{}",
            source.owner, source.name
        );

        let request = models::GetSourceStatisticsRequest {
            comment_filter: Some(models::CommentFilter::default()),
            ..Default::default()
        };

        let response = get_source_statistics(config, &source.owner, &source.name, request)
            .with_context(|| {
                format!(
                    "Failed to get statistics for source {}/{}",
                    source.owner, source.name
                )
            })?;

        source_stats.insert(source.id.clone(), *response.statistics);
    }

    Ok(source_stats)
}

/// Create printable source objects with optional bucket and statistics
fn create_printable_sources(
    sources: Vec<models::Source>,
    buckets_by_id: HashMap<String, models::Bucket>,
    source_stats: HashMap<String, models::SourceStatistics>,
) -> Vec<PrintableSource> {
    sources
        .into_iter()
        .map(|source| {
            let bucket = source
                .bucket_id
                .as_ref()
                .and_then(|id| buckets_by_id.get(id))
                .cloned();
            let stats = source_stats.get(&source.id).cloned();

            PrintableSource {
                source,
                bucket,
                stats,
            }
        })
        .collect()
}
