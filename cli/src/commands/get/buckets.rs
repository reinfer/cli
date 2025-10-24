use std::collections::HashMap;

use anyhow::{Context, Result};
use log::info;
use structopt::StructOpt;

use openapi::{
    apis::{
        buckets_api::{get_all_buckets, get_bucket_statistics},
        configuration::Configuration,
    },
    models::Bucket,
};

use crate::{
    printer::{PrintableBucket, Printer},
    utils::{resolve_bucket, BucketIdentifier},
};

#[derive(Debug, StructOpt)]
pub struct GetBucketsArgs {
    #[structopt(name = "bucket")]
    /// If specified, only list this bucket (name or id)
    bucket: Option<BucketIdentifier>,

    #[structopt(long = "stats")]
    /// Whether to include bucket statistics in response
    include_stats: bool,
}

/// Retrieve buckets with optional statistics
pub fn get(config: &Configuration, args: &GetBucketsArgs, printer: &Printer) -> Result<()> {
    let GetBucketsArgs {
        bucket,
        include_stats,
    } = args;

    let buckets = if let Some(bucket_identifier) = bucket {
        vec![resolve_bucket(config, bucket_identifier)?]
    } else {
        get_all_buckets_sorted(config)?
    };

    let bucket_stats = if *include_stats {
        collect_bucket_statistics(config, &buckets)?
    } else {
        HashMap::new()
    };

    let printable_buckets = create_printable_buckets(buckets, bucket_stats);
    printer.print_resources(&printable_buckets)
}

/// Retrieve all buckets, sorted by owner and name
fn get_all_buckets_sorted(config: &Configuration) -> Result<Vec<Bucket>> {
    let mut buckets = get_all_buckets(config)
        .context("Failed to list buckets")?
        .buckets;

    buckets.sort_unstable_by(|lhs, rhs| (&lhs.owner, &lhs.name).cmp(&(&rhs.owner, &rhs.name)));

    Ok(buckets)
}

/// Collect statistics for all provided buckets
fn collect_bucket_statistics(
    config: &Configuration,
    buckets: &[Bucket],
) -> Result<HashMap<String, openapi::models::BucketStatistics>> {
    let mut bucket_stats = HashMap::new();

    for bucket in buckets {
        info!(
            "Getting statistics for bucket {}/{}",
            bucket.owner, bucket.name
        );

        let stats = get_bucket_statistics(config, &bucket.owner, &bucket.name)
            .with_context(|| {
                format!(
                    "Failed to get statistics for bucket {}/{}",
                    bucket.owner, bucket.name
                )
            })?
            .statistics;

        bucket_stats.insert(bucket.id.clone(), *stats);
    }

    Ok(bucket_stats)
}

/// Create printable bucket objects with optional statistics
fn create_printable_buckets(
    buckets: Vec<Bucket>,
    bucket_stats: HashMap<String, openapi::models::BucketStatistics>,
) -> Vec<PrintableBucket> {
    buckets
        .into_iter()
        .map(|bucket| {
            let stats = bucket_stats.get(&bucket.id).cloned();
            PrintableBucket { bucket, stats }
        })
        .collect()
}
