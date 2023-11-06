use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use once_cell::sync::Lazy;
use reinfer_client::{BucketFullName, BucketType, Client, NewBucket, TransformTag};
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

    #[structopt(long = "transform-tag")]
    /// Set the transform tag of the new bucket. You will be given this value
    /// by a Re:infer engineer.
    transform_tag: Option<TransformTag>,
}

static DEFAULT_TRANSFORM_TAG: Lazy<TransformTag> =
    Lazy::new(|| TransformTag("generic.0.CONVKER5".to_string()));

pub fn create(client: &Client, args: &CreateBucketArgs, printer: &Printer) -> Result<()> {
    let CreateBucketArgs {
        name,
        title,
        bucket_type,
        transform_tag,
    } = args;

    let transform_tag = &if let Some(transform_tag) = transform_tag {
        transform_tag.clone()
    } else {
        DEFAULT_TRANSFORM_TAG.clone()
    };

    let bucket = client
        .create_bucket(
            name,
            NewBucket {
                title: title.as_deref(),
                bucket_type: *bucket_type,
                transform_tag,
            },
        )
        .context("Operation to create a bucket has failed")?;
    info!(
        "New bucket `{}` [id: {}] created successfully",
        bucket.full_name(),
        bucket.id
    );
    printer.print_resources(&[bucket])?;
    Ok(())
}
