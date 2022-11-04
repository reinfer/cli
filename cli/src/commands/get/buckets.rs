use anyhow::{Context, Result};
use reinfer_client::{BucketIdentifier, Client};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetBucketsArgs {
    #[structopt(name = "bucket")]
    /// If specified, only list this bucket (name or id)
    bucket: Option<BucketIdentifier>,
}

pub async fn get(client: &Client, args: &GetBucketsArgs, printer: &Printer) -> Result<()> {
    let GetBucketsArgs { bucket } = args;
    let buckets = if let Some(bucket) = bucket {
        vec![client
            .get_bucket(bucket.clone())
            .await
            .context("Operation to list buckets has failed.")?]
    } else {
        let mut buckets = client
            .get_buckets()
            .await
            .context("Operation to list buckets has failed.")?;
        buckets.sort_unstable_by(|lhs, rhs| {
            (&lhs.owner.0, &lhs.name.0).cmp(&(&rhs.owner.0, &rhs.name.0))
        });
        buckets
    };
    printer.print_resources(&buckets)
}
