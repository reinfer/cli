use crate::printer::Printer;
use crate::utils::FullName as BucketFullName;
use anyhow::{Context, Result};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        buckets_api::create_bucket,
    },
    models::BucketType,
};
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

pub fn create(config: &Configuration, args: &CreateBucketArgs, printer: &Printer) -> Result<()> {
    let CreateBucketArgs {
        name,
        title,
        bucket_type,
    } = args;

    let bucket_new = models::BucketNew {
        title,
        bucket_type: Some(bucket_type),
    };

    let create_request = models::CreateBucketRequest {
        bucket: Box::new(bucket_new),
    };

    let response = create_bucket(config, name.owner(), name.name(), create_request)
        .context("Operation to create a bucket has failed")?;

    let bucket = *response.bucket;
    
    info!(
        "New bucket `{}/{}` [id: {}] created successfully",
        bucket.owner,
        bucket.name,
        bucket.id,
    );
    printer.print_resources(&[bucket])?;
    Ok(())
}
