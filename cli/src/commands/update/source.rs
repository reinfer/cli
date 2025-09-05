use crate::printer::Printer;
use crate::utils::{BucketIdentifier, SourceIdentifier, transform_tag::TransformTag};
use anyhow::{Context, Result};
use log::info;
use structopt::StructOpt;

use openapi::{
    apis::{
        configuration::Configuration,
        sources_api::{update_source, get_source, get_source_by_id},
        buckets_api::get_bucket,
    },
    models,
};

// Custom types for CLI arguments

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

pub fn update(config: &Configuration, args: &UpdateSourceArgs, printer: &Printer) -> Result<()> {
    let UpdateSourceArgs {
        source,
        title,
        description,
        should_translate,
        bucket,
        transform_tag,
        detach_bucket,
    } = args;

    // Get existing source to determine owner/name for update
    let existing_source = match source.to_owned() {
        SourceIdentifier::FullName(full_name) => {
            let response = get_source(config, full_name.owner(), full_name.name())
                .context("Fetching existing source")?;
            *response.source
        }
        SourceIdentifier::Id(id) => {
            let response = get_source_by_id(config, &id)
                .context("Fetching existing source by ID")?;
            *response.source
        }
    };

    // Handle bucket ID resolution
    let bucket_id = match bucket.to_owned() {
        Some(BucketIdentifier::Id(bucket_id)) => Some(Some(bucket_id)),
        Some(BucketIdentifier::FullName(_)) => {
            // Get owner and name from the bucket identifier
            let owner = bucket.as_ref().unwrap().owner()
                .ok_or_else(|| anyhow::anyhow!("Failed to extract owner from bucket identifier"))?;
            let bucket_name = bucket.as_ref().unwrap().name()
                .ok_or_else(|| anyhow::anyhow!("Failed to extract name from bucket identifier"))?;
            
            // Get bucket by full name to get the ID
            let bucket_response = get_bucket(config, owner, bucket_name)
                .context("Failed to get bucket by full name")?;
            Some(Some(bucket_response.bucket.id))
        }
        None => {
            // Use existing bucket ID if not detaching
            if *detach_bucket {
                None
            } else {
                existing_source.bucket_id.map(|id| Some(id))
            }
        }
    };

    // Create the source update request
    let source_update = models::SourceUpdate {
        _kind: None, // Don't update the kind
        language: None, // Don't update the language
        title: title.clone(),
        description: description.clone(),
        should_translate: *should_translate,
        sensitive_properties: None,
        bucket_id,
        email_transform_tag: transform_tag.as_ref().map(|t| t.as_str().to_string()),
        email_transform_version: None,
    };

    let update_request = models::UpdateSourceRequest {
        source: Box::new(source_update),
    };

    // Call the generated API
    let response = update_source(config, &existing_source.owner, &existing_source.name, update_request)
        .context("Failed to update source")?;

    let updated_source = *response.source;
    
    info!(
        "Source `{}/{}` [id: {}] updated successfully",
        updated_source.owner,
        updated_source.name,
        updated_source.id
    );
    
    printer.print_resources(&[updated_source])?;
    Ok(())
}
