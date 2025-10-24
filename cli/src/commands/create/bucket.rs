//! Commands for creating buckets

// External crate imports
use anyhow::{Context, Result};
use log::info;
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{buckets_api::create_bucket, configuration::Configuration},
    models::{bucket_new::BucketType, BucketNew, CreateBucketRequest},
};

// Local crate imports
use crate::{printer::Printer, utils::FullName as BucketFullName};

/// Command line arguments for creating a bucket
#[derive(Debug, StructOpt)]
pub struct CreateBucketArgs {
    #[structopt(name = "bucket-name")]
    /// Full name of the new bucket <owner>/<name>
    name: BucketFullName,

    #[structopt(long = "title")]
    /// Set the title of the new bucket
    title: Option<String>,

    #[structopt(default_value = "emails", long = "type")]
    /// Set the type of the new bucket. Currently, this must be "emails".
    bucket_type: String,
}

/// Create a new bucket with the specified parameters
pub fn create(config: &Configuration, args: &CreateBucketArgs, printer: &Printer) -> Result<()> {
    let CreateBucketArgs {
        name,
        title,
        bucket_type,
    } = args;

    let parsed_bucket_type = match bucket_type.as_str() {
        "emails" => BucketType::Emails,
        _ => anyhow::bail!(
            "Invalid bucket type '{}'. Currently, only 'emails' is supported.",
            bucket_type
        ),
    };

    let bucket_new = BucketNew {
        title: title.clone(),
        bucket_type: Some(parsed_bucket_type),
    };

    let create_request = CreateBucketRequest {
        bucket: Box::new(bucket_new),
    };

    let response = create_bucket(config, name.owner(), name.name(), create_request)
        .context("Operation to create a bucket has failed")?;

    let bucket = *response.bucket;

    info!(
        "New bucket `{}/{}` [id: {}] created successfully",
        bucket.owner, bucket.name, bucket.id,
    );
    printer.print_resources(&[bucket])?;
    Ok(())
}
