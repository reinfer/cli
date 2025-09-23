use std::collections::HashMap;

use anyhow::{Context, Result};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        buckets_api::{get_bucket, get_buckets, get_bucket_by_id, get_bucket_statistics},
    },
    models::BucketIdentifier,
};
use structopt::StructOpt;

use crate::printer::{PrintableBucket, Printer};

#[derive(Debug, StructOpt)]
pub struct GetBucketsArgs {
    #[structopt(name = "bucket")]
    /// If specified, only list this bucket (name or id)
    bucket: Option<BucketIdentifier>,

    #[structopt(long = "stats")]
    /// Whether to include bucket statistics in response
    include_stats: bool,
}

pub fn get(config: &Configuration, args: &GetBucketsArgs, printer: &Printer) -> Result<()> {
    let GetBucketsArgs {
        bucket,
        include_stats,
    } = args;

    let buckets = if let Some(bucket) = bucket {
        vec![match bucket {
            BucketIdentifier::Id(id) => get_bucket_by_id(config, id)
                .context("Operation to get bucket by ID has failed.")?
                .bucket,
            BucketIdentifier::FullName(full_name) => get_bucket(config, full_name.owner(), full_name.name())
                .context("Operation to get bucket has failed.")?
                .bucket,
        }]
    } else {
        let mut buckets = get_buckets(config)
            .context("Operation to list buckets has failed.")?
            .buckets;
        buckets.sort_unstable_by(|lhs, rhs| {
            (&lhs.owner, &lhs.name).cmp(&(&rhs.owner, &rhs.name))
        });
        buckets
    };

    let mut bucket_stats: HashMap<_, _> = HashMap::new();

    if *include_stats {
        buckets.iter().try_for_each(|bucket| -> Result<()> {
            info!("Getting statistics for bucket {}", bucket.full_name().0);
            let stats = get_bucket_statistics(config, &bucket.owner, &bucket.name)
                .context("Could not get statistics for bucket")?
                .statistics;

            bucket_stats.insert(bucket.id.clone(), stats);
            Ok(())
        })?;
    }

    let printable_buckets: Vec<PrintableBucket> = buckets
        .into_iter()
        .map(|bucket| {
            let stats = bucket_stats.get(&bucket.id).cloned();
            PrintableBucket { bucket, stats }
        })
        .collect();

    printer.print_resources(&printable_buckets)
}
