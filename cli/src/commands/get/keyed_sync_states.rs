use anyhow::{Context, Result};
use openapi::{
    apis::{
        buckets_api::{get_bucket, get_bucket_by_id, list_keyed_sync_states},
        configuration::Configuration,
    },
    models::Bucket,
};
use structopt::StructOpt;

use crate::printer::Printer;
use crate::utils::resource_identifier::BucketIdentifier;

#[derive(Debug, StructOpt)]
pub struct GetKeyedSyncStatesArgs {
    #[structopt(name = "bucket")]
    /// The bucket to get keyed sync states for
    bucket: BucketIdentifier,
}

pub fn get(config: &Configuration, args: &GetKeyedSyncStatesArgs, printer: &Printer) -> Result<()> {
    let GetKeyedSyncStatesArgs { bucket } = args;

    let bucket = match bucket {
        BucketIdentifier::Id(bucket_id) => {
            let response = get_bucket_by_id(config, bucket_id)
                .context("Failed to get bucket by ID")?;
            response.bucket
        }
        BucketIdentifier::FullName(full_name) => {
            let response = get_bucket(config, full_name.owner(), full_name.name())
                .context("Failed to get bucket by name")?;
            response.bucket
        }
    };

    let response = list_keyed_sync_states(config, &bucket.id)
        .context("Failed to get keyed sync states")?;

    printer.print_resources(&response.keyed_sync_states)
}
