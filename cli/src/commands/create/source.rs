use crate::printer::Printer;
use crate::utils::{BucketIdentifier, FullName, TransformTag};
use anyhow::{Context, Result};
use log::info;
use structopt::StructOpt;

use openapi::{
    apis::{
        configuration::Configuration,
        sources_api::create_source,
        buckets_api::get_bucket,
    },
    models,
};

#[derive(Debug, StructOpt)]
pub struct CreateSourceArgs {
    #[structopt(name = "source-name")]
    /// Full name of the new source <owner>/<name>
    name: FullName,

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
    kind: Option<models::SourceKind>,

    #[structopt(long = "transform-tag")]
    /// Set the transform tag of the new source
    transform_tag: Option<TransformTag>,
}

pub fn create(config: &Configuration, args: &CreateSourceArgs, printer: &Printer) -> Result<()> {
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
        None => None,
    };

    // Convert language string to Language enum if provided
    let language_enum = language.as_ref().map(|lang_str| {
        match lang_str.as_str() {
            "en" => models::Language::En,
            "de" => models::Language::De,
            "xlm" => models::Language::Xlm,
            _ => return Err(anyhow::anyhow!("Unsupported language: '{}'. Supported languages: en, de, xlm", lang_str)),
        }
    }).transpose()?;

    // Create the source update request
    let source_update = models::SourceUpdate {
        _kind: kind.clone(),
        language: language_enum,
        title: title.clone(),
        description: description.clone(),
        should_translate: *should_translate,
        sensitive_properties: None,
        bucket_id,
        email_transform_tag: transform_tag.as_ref().map(|t| t.as_str().to_string()),
        email_transform_version: None,
    };

    let create_request = models::CreateSourceRequest {
        source: Box::new(source_update),
    };

    // Call the generated API
    let response = create_source(config, name.owner(), name.name(), create_request)
        .context("Failed to create source")?;

    let source = *response.source;
    
    info!(
        "New source `{}/{}` [id: {}] created successfully",
        source.owner,
        source.name,
        source.id
    );
    
    printer.print_resources(&[source])?;
    Ok(())
}
