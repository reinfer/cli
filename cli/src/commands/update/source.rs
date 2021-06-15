use anyhow::{Context, Result};
use log::info;
use reinfer_client::{BucketId, BucketIdentifier, Client, SourceIdentifier, UpdateSource};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct UpdateSourceArgs {
    #[structopt(name = "source")]
    /// Id or full name of the source to update
    source: SourceIdentifier,

    #[structopt(long = "title")]
    /// Set the title of the source
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the source
    description: Option<String>,

    #[structopt(long = "should-translate")]
    /// Enable translation for the source
    should_translate: Option<bool>,

    #[structopt(long = "bucket")]
    /// Bucket to pull emails from.
    bucket: Option<BucketIdentifier>,
}

pub fn update(client: &Client, args: &UpdateSourceArgs) -> Result<()> {
    let UpdateSourceArgs {
        ref source,
        ref title,
        ref description,
        ref should_translate,
        ref bucket,
    } = *args;

    let bucket_id = bucket
        .as_ref()
        .map(Clone::clone)
        .map(|bucket| -> Result<BucketId> {
            Ok(match bucket {
                BucketIdentifier::Id(bucket_id) => bucket_id,
                BucketIdentifier::FullName(_) => {
                    client
                        .get_bucket(bucket)
                        .context("Fetching bucket for id.")?
                        .id
                }
            })
        })
        .transpose()?;

    let source_full_name = match source.to_owned() {
        SourceIdentifier::FullName(name) => name,
        source @ SourceIdentifier::Id(_) => client
            .get_source(source)
            .context("Fetching source id.")?
            .full_name(),
    };

    let source = client
        .update_source(
            &source_full_name,
            UpdateSource {
                title: title.as_ref().map(|title| title.as_str()),
                description: description.as_ref().map(|description| description.as_str()),
                should_translate: *should_translate,
                bucket_id,
                sensitive_properties: None,
            },
        )
        .context("Operation to update a source has failed")?;
    info!(
        "Source `{}` [id: {}] updated successfully",
        source.full_name().0,
        source.id.0
    );
    Ok(())
}
