use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{BucketFullName, BucketType, Client, NewBucket};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateBucketArgs {
    #[structopt(name = "bucket-name")]
    /// Full name of the new bucket <owner>/<name>
    name: BucketFullName,

    #[structopt(long = "title")]
    /// Set the title of the new bucket
    title: Option<String>,

    #[structopt(default_value, long = "type")]
    /// Set the type of the new bucket. Currently, this must be "emails".
    bucket_type: BucketType,
}

pub fn create(client: &Client, args: &CreateBucketArgs, printer: &Printer) -> Result<()> {
    let CreateBucketArgs {
        name,
        title,
        bucket_type,
    } = args;

    let bucket = client
        .create_bucket(
            name,
            NewBucket {
                title: title.as_deref(),
                bucket_type: *bucket_type,
            },
        )
        .context("Operation to create a bucket has failed")?;
    info!(
        "New bucket `{}` [id: {}] created successfully",
        bucket.full_name(),
        bucket.id,
    );
    printer.print_resources(&[bucket])?;
    Ok(())
}
