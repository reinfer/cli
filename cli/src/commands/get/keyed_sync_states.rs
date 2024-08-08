use anyhow::Result;
use reinfer_client::{BucketIdentifier, Client};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetKeyedSyncStatesArgs {
    #[structopt(name = "bucket")]
    /// The bucket to get keyed sync states for
    bucket: BucketIdentifier,
}

pub fn get(client: &Client, args: &GetKeyedSyncStatesArgs, printer: &Printer) -> Result<()> {
    let GetKeyedSyncStatesArgs { bucket } = args;

    let bucket = client.get_bucket(bucket.clone())?;

    let keyed_sync_states = client.get_keyed_sync_states(&bucket.id)?;

    printer.print_resources(&keyed_sync_states)
}
