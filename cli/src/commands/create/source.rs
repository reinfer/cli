use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    BucketIdentifier, Client, NewSource, SourceFullName, SourceKind, TransformTag,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateSourceArgs {
    #[structopt(name = "source-name")]
    /// Full name of the new source <owner>/<name>
    name: SourceFullName,

    #[structopt(long = "title")]
    /// Set the title of the new source
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the new source
    description: Option<String>,

    #[structopt(long = "language")]
    /// Set the language of the new source
    language: Option<String>,

    #[structopt(long = "should-translate")]
    /// Enable translation for the source
    should_translate: Option<bool>,

    #[structopt(long = "bucket")]
    /// Bucket to pull emails from.
    bucket: Option<BucketIdentifier>,

    #[structopt(long = "kind")]
    /// Set the kind of the new source
    kind: Option<SourceKind>,

    #[structopt(long = "transform-tag")]
    /// Set the transform tag of the new source
    transform_tag: Option<TransformTag>,
}

pub fn create(client: &Client, args: &CreateSourceArgs, printer: &Printer) -> Result<()> {
    let CreateSourceArgs {
        name,
        title,
        description,
        language,
        should_translate,
        bucket,
        kind,
        transform_tag,
    } = args;

    let bucket_id = match bucket.to_owned() {
        Some(BucketIdentifier::Id(bucket_id)) => Some(bucket_id),
        Some(full_name @ BucketIdentifier::FullName(_)) => Some(
            client
                .get_bucket(full_name)
                .context("Fetching bucket for id.")?
                .id,
        ),
        None => None,
    };

    let source = client
        .create_source(
            name,
            NewSource {
                title: title.as_deref(),
                description: description.as_deref(),
                language: language.as_deref(),
                should_translate: *should_translate,
                bucket_id,
                sensitive_properties: None,
                kind: kind.as_ref(),
                transform_tag: transform_tag.as_ref(),
            },
        )
        .context("Operation to create a source has failed")?;
    info!(
        "New source `{}` [id: {}] created successfully",
        source.full_name().0,
        source.id.0
    );
    printer.print_resources(&[source])?;
    Ok(())
}
