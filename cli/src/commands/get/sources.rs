use anyhow::{Context, Result};
use log::info;
use std::collections::HashMap;
use std::str::FromStr;
use structopt::StructOpt;
use crate::printer::{PrintableSource, Printer};
use crate::utils::SourceIdentifier;

use openapi::{
    apis::{
        configuration::Configuration,
        sources_api::{
            get_all_sources,
            get_source,
            get_source_by_id,
            get_source_statistics,
        },
        buckets_api::get_all_buckets,
    },
    models};

#[derive(Debug, StructOpt)]
pub struct GetSourcesArgs {
    /// If specified, only list this source (name or id)
    #[structopt(name = "source", parse(try_from_str = SourceIdentifier::from_str))]
    source: Option<SourceIdentifier>,
    /// Whether to include source statistics in response
    #[structopt(long = "stats")]
    include_stats: bool,
}

pub fn get(config: &Configuration, args: &GetSourcesArgs, printer: &Printer) -> Result<()> {
    let GetSourcesArgs { 
        source,
        include_stats
    } = args;

    let mut sources: Vec<models::Source> = match source {
        Some(source_identifier) => {
            let resp = match source_identifier {
                SourceIdentifier::FullName(full) => {
                    get_source(config, full.owner(), full.name())
                        .context("get source by full name")?
                }
                SourceIdentifier::Id(id) => {
                    get_source_by_id(config, &id)
                        .context("get source by id")?
                }
            };
            vec![*resp.source]
        }
        None => {
            let resp = get_all_sources(config)
                .context("list sources")?;
            resp.sources
        }
    };

    sources.sort_unstable_by(|a, b| (&a.owner, &a.name).cmp(&(&b.owner, &b.name)));

 
    let mut buckets_by_id: HashMap<String, models::Bucket> = {
        let resp = get_all_buckets(config)
            .context("list buckets")?;
        resp.buckets.into_iter().map(|b| (b.id.clone(), b)).collect()
    };

    let mut source_stats: HashMap<_, _> = HashMap::new();
    if *include_stats {
        for s in &sources {
            info!("Getting statistics for source {}/{}", s.owner, s.name);
            let req = models::GetSourceStatisticsRequest {
                comment_filter: Some(models::CommentFilter::default()),
                ..Default::default()
            };

            let resp = get_source_statistics(
                config,
                &s.owner,
                &s.name,
                req,
            )
            .context("Could not get statistics for source")?;

            let stats: models::SourceStatistics = *resp.statistics;

            // Index by source id (String)
            source_stats.insert(s.id.clone(), stats);
        }
    }

    let printable_sources: Vec<PrintableSource> = sources
        .into_iter()
        .map(|src| {
            let bucket = src.bucket_id.as_ref().and_then(|id| buckets_by_id.get(id)).cloned();
            let stats  = source_stats.get(&src.id).cloned(); 
            PrintableSource { source: src, bucket, stats }
        })
        .collect();

    printer.print_resources(&printable_sources)
}
