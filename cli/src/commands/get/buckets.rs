use std::collections::HashMap;

use anyhow::{Context, Result};
use log::info;
use reinfer_client::{BucketIdentifier, Client};
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

pub fn get(client: &Client, args: &GetBucketsArgs, printer: &Printer) -> Result<()> {
    let GetBucketsArgs {
        bucket,
        include_stats,
    } = args;

    let buckets = if let Some(bucket) = bucket {
        vec![client
            .get_bucket(bucket.clone())
            .context("Operation to list buckets has failed.")?]
    } else {
        let mut buckets = client
            .get_buckets()
            .context("Operation to list buckets has failed.")?;
        buckets.sort_unstable_by(|lhs, rhs| {
            (&lhs.owner.0, &lhs.name.0).cmp(&(&rhs.owner.0, &rhs.name.0))
        });
        buckets
    };

    let mut bucket_stats: HashMap<_, _> = HashMap::new();

    if *include_stats {
        buckets.iter().try_for_each(|bucket| -> Result<()> {
            info!("Getting statistics for bucket {}", bucket.full_name().0);
            let stats = client
                .get_bucket_statistics(&bucket.full_name())
                .context("Could not get statistics for bucket")?;

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
