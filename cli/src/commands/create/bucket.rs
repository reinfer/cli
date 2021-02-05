use anyhow::{Context, Result};
use log::info;
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

pub fn create(client: &Client, args: &CreateBucketArgs) -> Result<()> {
    let CreateBucketArgs {
        ref name,
        ref title,
        bucket_type,
        ref transform_tag,
    } = *args;

    let bucket = client
        .create_bucket(
            name,
            NewBucket {
                title: title.as_ref().map(|title| title.as_str()),
                bucket_type,
                transform_tag: transform_tag.as_ref(),
            },
        )
        .context("Operation to create a bucket has failed")?;
    info!(
        "New bucket `{}` [id: {}] created successfully",
        bucket.full_name(),
        bucket.id
    );
    Ok(())
}
