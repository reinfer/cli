use anyhow::{Context, Result};
use log::info;
use reinfer_client::{resources::source::StatisticsRequestParams, Client, SourceIdentifier};
use std::collections::HashMap;
use structopt::StructOpt;

use crate::printer::{PrintableSource, Printer};

#[derive(Debug, StructOpt)]
pub struct GetSourcesArgs {
    #[structopt(name = "source")]
    /// If specified, only list this source (name or id)
    source: Option<SourceIdentifier>,

    #[structopt(long = "stats")]
    /// Whether to include source statistics in response
    include_stats: bool,
}

pub fn get(client: &Client, args: &GetSourcesArgs, printer: &Printer) -> Result<()> {
    let GetSourcesArgs {
        source,
        include_stats,
    } = args;

    let sources = if let Some(source) = source {
        vec![client
            .get_source(source.clone())
            .context("Operation to list sources has failed.")?]
    } else {
        let mut sources = client
            .get_sources()
            .context("Operation to list sources has failed.")?;
        sources.sort_unstable_by(|lhs, rhs| {
            (&lhs.owner.0, &lhs.name.0).cmp(&(&rhs.owner.0, &rhs.name.0))
        });
        sources
    };

    let buckets: HashMap<_, _> = client
        .get_buckets()
        .context("Operation to list buckets has failed.")?
        .into_iter()
        .map(|bucket| (bucket.id.clone(), bucket))
        .collect();

    let mut source_stats: HashMap<_, _> = HashMap::new();
    if *include_stats {
        sources.iter().try_for_each(|source| -> Result<()> {
            info!("Getting statistics for source {}", source.full_name().0);
            let stats = client
                .get_source_statistics(
                    &source.full_name(),
                    &StatisticsRequestParams {
                        comment_filter: Default::default(),
                    },
                )
                .context("Could not get statistics for source")?;

            source_stats.insert(source.id.clone(), stats);
            Ok(())
        })?;
    };

    let printable_sources: Vec<PrintableSource> = sources
        .into_iter()
        .map(|source| {
            let bucket = source
                .bucket_id
                .as_ref()
                .and_then(|id| buckets.get(id))
                .cloned();

            let stats = source_stats.get(&source.id).cloned();
            PrintableSource {
                source,
                bucket,
                stats,
            }
        })
        .collect();

    printer.print_resources(&printable_sources)
}
