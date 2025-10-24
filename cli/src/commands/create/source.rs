//! Commands for creating sources with comprehensive configuration options
//!
//! This module provides functionality to:
//! - Create sources with various types (email, comment, call, chat, etc.)
//! - Configure language settings and translation options
//! - Associate sources with buckets for email processing
//! - Set transform tags for email processing pipelines

// External crate imports
use anyhow::Result;
use log::info;
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{configuration::Configuration, sources_api::create_source},
    models,
};

// Local crate imports
use crate::{
    printer::Printer,
    utils::{types::identifiers::BucketIdentifier, types::transform::TransformTag, FullName},
};

/// Command line arguments for creating a source with full configuration options
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
    kind: Option<String>,

    #[structopt(long = "transform-tag")]
    /// Set the transform tag of the new source
    transform_tag: Option<TransformTag>,
}

/// Create a new source with the specified configuration
///
/// This function handles:
/// - Bucket resolution from name or ID
/// - Language and source kind validation
/// - Transform tag configuration
/// - Comprehensive error handling with detailed messages
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

    let bucket_id = match bucket {
        Some(bucket_id) => {
            let bucket = crate::utils::resolve_bucket(config, bucket_id)?;
            Some(bucket.id)
        }
        None => None,
    };

    let language_enum = match language {
        Some(lang_str) => {
            let language_enum = match lang_str.as_str() {
                "en" => models::Language::En,
                "de" => models::Language::De,
                "xlm" => models::Language::Xlm,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported language: '{}'. Supported languages: en, de, xlm",
                        lang_str
                    ))
                }
            };
            Some(language_enum)
        }
        None => None,
    };

    let kind_enum = match kind {
        Some(kind_str) => {
            let kind_enum = match kind_str.as_str() {
                // Primary source types for communications mining
                "email" => models::SourceKind::Unknown,
                "comment" => models::SourceKind::Unknown,
                // OpenAPI generated variants
                "call" => models::SourceKind::Call,
                "chat" => models::SourceKind::Chat,
                "unknown" => models::SourceKind::Unknown,
                "ixp_design" => models::SourceKind::IxpDesign,
                "ixp_runtime" => models::SourceKind::IxpRuntime,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid source kind: '{}'. Supported kinds: email, comment, call, chat, unknown, ixp_design, ixp_runtime",
                        kind_str
                    ))
                }
            };
            Some(kind_enum)
        }
        None => None,
    };

    // Create the source request
    let source_update = models::SourceUpdate {
        title: title.clone(),
        description: description.clone(),
        language: language_enum,
        should_translate: *should_translate,
        bucket_id: bucket_id.map(Some), // Convert Option<String> to Option<Option<String>>
        sensitive_properties: None,
        _kind: kind_enum,
        email_transform_tag: transform_tag.as_ref().map(|t| t.as_str().to_string()),
        email_transform_version: None,
    };

    let create_request = models::CreateSourceRequest {
        source: Box::new(source_update),
    };

    // Call the generated API
    let response =
        create_source(config, name.owner(), name.name(), create_request).map_err(|e| {
            match e {
                openapi::apis::Error::ResponseError(response_content) => {
                    // Try to parse the error response to get the detailed message
                    let detailed_error = if let Ok(error_resp) =
                        serde_json::from_str::<openapi::models::ErrorResponse>(
                            &response_content.content,
                        ) {
                        error_resp.message
                    } else {
                        response_content.content
                    };
                    anyhow::anyhow!(
                        "Operation to create a source has failed: {}",
                        detailed_error
                    )
                }
                _ => anyhow::anyhow!("Operation to create a source has failed: {}", e),
            }
        })?;

    let source = *response.source;

    info!(
        "New source `{}/{}` [id: {}] created successfully",
        source.owner, source.name, source.id
    );
    printer.print_resources(&[source])?;
    Ok(())
}
