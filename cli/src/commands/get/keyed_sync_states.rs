use anyhow::{Context, Result};
use structopt::StructOpt;

use openapi::apis::{buckets_api::list_keyed_sync_states, configuration::Configuration};

use crate::{
    printer::Printer,
    utils::{resolve_bucket, BucketIdentifier},
};

#[derive(Debug, StructOpt)]
pub struct GetKeyedSyncStatesArgs {
    #[structopt(name = "bucket")]
    /// The bucket to get keyed sync states for
    bucket: BucketIdentifier,
}

/// Retrieve keyed sync states for a bucket
pub fn get(config: &Configuration, args: &GetKeyedSyncStatesArgs, printer: &Printer) -> Result<()> {
    let GetKeyedSyncStatesArgs { bucket } = args;

    let bucket = resolve_bucket(config, bucket)?;
    let keyed_sync_states = fetch_keyed_sync_states(config, &bucket.id)?;

    printer.print_resources(&keyed_sync_states)
}

/// Fetch keyed sync states for a bucket
fn fetch_keyed_sync_states(
    config: &Configuration,
    bucket_id: &str,
) -> Result<Vec<openapi::models::ListKeyedSyncStatesResponseKeyedSyncStatesInner>> {
    let response = list_keyed_sync_states(config, bucket_id)
        .with_context(|| format!("Failed to get keyed sync states for bucket {}", bucket_id))?;

    Ok(response.keyed_sync_states)
}
