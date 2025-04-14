use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{BucketIdentifier, Client, SourceIdentifier, TransformTag, UpdateSource};
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

    #[structopt(long = "transform-tag")]
    /// Set the transform tag of the source
    transform_tag: Option<TransformTag>,

    #[structopt(long = "detach-bucket")]
    /// Detach bucket from source
    detach_bucket: bool,
}

pub fn update(client: &Client, args: &UpdateSourceArgs, printer: &Printer) -> Result<()> {
    let UpdateSourceArgs {
        source,
        title,
        description,
        should_translate,
        bucket,
        transform_tag,
        detach_bucket,
    } = args;

    let existing_source = client
        .get_source(source.to_owned())
        .context("Fetching existing source")?;

    let bucket_id = match bucket.to_owned() {
        Some(BucketIdentifier::Id(bucket_id)) => Some(bucket_id),
        Some(full_name @ BucketIdentifier::FullName(_)) => Some(
            client
                .get_bucket(full_name)
                .context("Fetching bucket for id.")?
                .id,
        ),
        None => existing_source.bucket_id.clone(),
    };

    let bucket_id = if *detach_bucket { None } else { bucket_id };

    let source_full_name = match source.to_owned() {
        SourceIdentifier::FullName(name) => name,
        SourceIdentifier::Id(_) => existing_source.full_name(),
    };

    let source = client
        .update_source(
            &source_full_name,
            UpdateSource {
                title: title.as_deref(),
                description: description.as_deref(),
                should_translate: *should_translate,
                bucket_id,
                sensitive_properties: None,
                transform_tag: transform_tag.as_ref(),
            },
        )
        .context("Operation to update a source has failed")?;
    info!(
        "Source `{}` [id: {}] updated successfully",
        source.full_name().0,
        source.id.0
    );
    printer.print_resources(&[source])?;
    Ok(())
}
