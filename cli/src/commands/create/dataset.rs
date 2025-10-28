//! Commands for creating datasets with comprehensive configuration options
//!
//! This module supports creating datasets with:
//! - Entity definitions and general fields
//! - Label definitions and label groups  
//! - Model family and feature flags (QoS, GenAI, etc.)
//! - Source associations and annotation copying

// Standard library imports
use std::str::FromStr;

// External crate imports
use anyhow::{anyhow, bail, Error, Result};
use log::info;
use serde::Deserialize;
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{configuration::Configuration, datasets_api::create_dataset},
    models::{
        CreateDatasetRequest, DatasetFlag, DatasetNew, EntityDefNew, GeneralFieldDefNew,
        LabelDefNew, LabelGroupNew,
    },
};

// Local crate imports
use crate::{
    printer::Printer,
    utils::{types::identifiers::SourceIdentifier, FullName as DatasetFullName},
};

/// Command line arguments for creating a dataset with full configuration options
#[derive(Debug, StructOpt)]
pub struct CreateDatasetArgs {
    #[structopt(name = "owner-name/dataset-name")]
    /// Full name of the new dataset <owner>/<name>
    name: DatasetFullName,

    #[structopt(long = "title")]
    /// Set the title of the new dataset
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the new dataset
    description: Option<String>,

    #[structopt(
        long = "has-sentiment",
        help = "Enable sentiment prediction for the dataset [default: false]"
    )]
    /// Enable sentiment prediction for the dataset
    has_sentiment: Option<bool>,

    #[structopt(short = "s", long = "source")]
    /// Names or ids of the sources in the dataset
    sources: Vec<SourceIdentifier>,

    #[structopt(short = "e", long = "entity-defs", default_value = "[]")]
    /// Entity defs to create at dataset creation, as json
    entity_defs: VecExt<EntityDefNew>,

    #[structopt(short = "g", long = "general-fields", default_value = "[]")]
    /// General Fields to create at dataset creation, as json
    general_fields: VecExt<GeneralFieldDefNew>,

    #[structopt(long = "label-defs", default_value = "[]")]
    /// Label defs to create at dataset creation, as json.
    /// Only used if label_groups is not provided.
    label_defs: VecExt<LabelDefNew>,

    #[structopt(long = "label-groups", default_value = "[]")]
    /// Label groups to create at dataset creation, as json
    label_groups: VecExt<LabelGroupNew>,

    #[structopt(long = "model-family")]
    /// Model family to use for the new dataset
    model_family: Option<String>,

    /// Dataset ID of the dataset to copy annotations from
    #[structopt(long = "copy-annotations-from")]
    copy_annotations_from: Option<String>,

    /// Whether the dataset should have QoS enabled
    #[structopt(long = "qos")]
    qos: Option<bool>,

    /// Whether to use the external llm
    #[structopt(long = "external-llm")]
    external_llm: Option<bool>,

    /// Whether to use generative ai features
    #[structopt(long = "gen-ai")]
    gen_ai: Option<bool>,

    /// Whether to use zero shot ai features
    #[structopt(long = "zero-shot")]
    zero_shot: Option<bool>,
}

/// Create a new dataset with the specified configuration
///
/// This function handles:
/// - Source ID resolution from names or IDs
/// - Dataset flag validation and setup
/// - Label definition vs label group precedence
/// - Comprehensive error handling with detailed messages
pub fn create(config: &Configuration, args: &CreateDatasetArgs, printer: &Printer) -> Result<()> {
    let CreateDatasetArgs {
        name,
        title,
        description,
        has_sentiment,
        sources,
        entity_defs,
        general_fields,
        label_defs,
        label_groups,
        model_family,
        copy_annotations_from,
        qos,
        external_llm,
        gen_ai,
        zero_shot,
    } = args;

    let mut source_ids = Vec::with_capacity(sources.len());
    for source_id in sources.iter() {
        let source = crate::utils::resolve_source(config, source_id)?;
        source_ids.push(source.id);
    }

    let dataset_flags = build_dataset_flags(*gen_ai, *external_llm, *zero_shot, *qos)?;

    // Unwrap the inner values, we only need the outer for argument parsing
    let entity_defs = &entity_defs.0;
    let general_fields = &general_fields.0;
    let label_groups = &label_groups.0;
    let label_defs = match (!&label_defs.0.is_empty(), !label_groups.is_empty()) {
        // if we only have label defs, then use them
        (true, false) => Some(&label_defs.0[..]),
        // otherwise, we either don't have defs or have groups, so don't use them
        _ => None,
    };
    let dataset_new = DatasetNew {
        title: title.clone(),
        description: description.clone(),
        source_ids: Some(source_ids),
        has_sentiment: Some(has_sentiment.unwrap_or(false)),
        entity_defs: if entity_defs.is_empty() {
            None
        } else {
            Some(entity_defs.clone())
        },
        general_fields: if general_fields.is_empty() {
            None
        } else {
            Some(general_fields.clone())
        },
        label_defs: label_defs.map(|label_defs| label_defs.to_vec()),
        label_groups: if label_groups.is_empty() {
            None
        } else {
            Some(label_groups.clone())
        },
        model_family: model_family.clone(),
        copy_annotations_from: copy_annotations_from.clone(),
        _dataset_flags: Some(dataset_flags),
        ..Default::default()
    };

    let create_request = CreateDatasetRequest {
        dataset: Box::new(dataset_new),
    };

    let response =
        create_dataset(config, name.owner(), name.name(), create_request).map_err(|e| {
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
                    anyhow!(
                        "Operation to create a dataset has failed: {}",
                        detailed_error
                    )
                }
                _ => anyhow!("Operation to create a dataset has failed: {}", e),
            }
        })?;

    let dataset = *response.dataset;
    info!(
        "New dataset `{}/{}` [id: {}] created successfully",
        dataset.owner, dataset.name, dataset.id,
    );
    printer.print_resources(&[dataset])?;
    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Build dataset flags based on feature selections with validation
fn build_dataset_flags(
    gen_ai: Option<bool>,
    external_llm: Option<bool>,
    zero_shot: Option<bool>,
    qos: Option<bool>,
) -> Result<Vec<DatasetFlag>> {
    // Validate flag combinations
    if external_llm.unwrap_or_default() && !gen_ai.unwrap_or_default() {
        bail!("External LLM can only be used if gen AI features are enabled. Please add `--gen-ai true`")
    }

    if zero_shot.unwrap_or_default() && !gen_ai.unwrap_or_default() {
        bail!(
            "Zero shot can only be used if gen AI features are enabled. Please add `--gen-ai true`"
        )
    }

    let mut dataset_flags = Vec::new();

    if gen_ai.unwrap_or_default() {
        dataset_flags.push(DatasetFlag::Gpt4)
    }

    if external_llm.unwrap_or_default() {
        dataset_flags.push(DatasetFlag::ExternalMoonLlm)
    }

    if zero_shot.unwrap_or_default() {
        dataset_flags.push(DatasetFlag::ZeroShotLabels)
    }

    if qos.unwrap_or_default() {
        dataset_flags.push(DatasetFlag::Qos)
    }

    Ok(dataset_flags)
}

// ============================================================================
// Utility Types
// ============================================================================

/// Wrapper type for JSON deserialization of Vec<T> from command line arguments
#[derive(Debug, Deserialize)]
struct VecExt<T>(pub Vec<T>);

/// Utility type for parsing JSON arrays from command line arguments
///
/// This enables parsing JSON arrays like `[{"name": "test"}]` directly from CLI args
impl<T: serde::de::DeserializeOwned> FromStr for VecExt<T> {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        serde_json::from_str(string).map_err(|source| {
            anyhow!(
                "Expected valid JSON array. Got: '{}', which failed because: '{}'",
                string.to_owned(),
                source
            )
        })
    }
}
