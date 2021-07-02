use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{BucketId, BucketIdentifier, Client, NewSource, SourceFullName};
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
}

pub fn create(client: &Client, args: &CreateSourceArgs, printer: &Printer) -> Result<()> {
    let CreateSourceArgs {
        ref name,
        ref title,
        ref description,
        ref language,
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

    let source = client
        .create_source(
            &name,
            NewSource {
                title: title.as_ref().map(|title| title.as_str()),
                description: description.as_ref().map(|description| description.as_str()),
                language: language.as_ref().map(|language| language.as_str()),
                should_translate: *should_translate,
                bucket_id,
                sensitive_properties: None,
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
