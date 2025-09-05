use crate::printer::Printer;
use crate::utils::{FullName as DatasetFullName, resource_identifier::SourceIdentifier};
use anyhow::{anyhow, bail, Context, Error, Result};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        datasets_api::create_dataset,
        sources_api::{get_source, get_source_by_id},
    },
    models::{
        CreateDatasetRequest, DatasetNew, DatasetFlag, EntityDefNew, GeneralFieldDefNew,
        LabelDefNew, LabelGroupNew,
    },
};
use serde::Deserialize;
use std::str::FromStr;
use structopt::StructOpt;

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

    let source_ids = {
        let mut source_ids = Vec::with_capacity(sources.len());
        for source in sources.iter() {
            let source_id = match source {
                SourceIdentifier::Id(id) => {
                    // It's a source ID, use it directly
                    id.clone()
                }
                SourceIdentifier::FullName(full_name) => {
                    // It's a source name, need to look it up
                    let response = get_source(config, full_name.owner(), full_name.name())
                        .context("Operation to get source has failed")?;
                    response.source.id
                }
            };
            source_ids.push(source_id);
        }
        source_ids
    };

    let get_dataset_flags = || -> Result<Vec<DatasetFlag>> {
        if external_llm.unwrap_or_default() && !gen_ai.unwrap_or_default() {
            bail!("External Llm can only be used if gen ai features are enabled. Please add `--gen-ai true`")
        }

        if zero_shot.unwrap_or_default() && !gen_ai.unwrap_or_default() {
            bail!("Zero shot can only be used if gen ai features are enabled. Please add `--gen-ai true`")
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
    };

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
        label_defs: if let Some(label_defs) = label_defs {
            Some(label_defs.clone())
        } else {
            None
        },
        label_groups: if label_groups.is_empty() {
            None
        } else {
            Some(label_groups.clone())
        },
        model_family: model_family.clone(),
        copy_annotations_from: copy_annotations_from.clone(),
        _dataset_flags: Some(get_dataset_flags()?),
        ..Default::default()
    };

    let create_request = CreateDatasetRequest {
        dataset: Box::new(dataset_new),
    };

    let response = create_dataset(config, name.owner(), name.name(), create_request)
        .context("Operation to create a dataset has failed.")?;

    let dataset = *response.dataset;
    info!(
        "New dataset `{}/{}` [id: {}] created successfully",
        dataset.owner,
        dataset.name,
        dataset.id,
    );
    printer.print_resources(&[dataset])?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct VecExt<T>(pub Vec<T>);

/// Utility type for foreign trait interactions.
///
/// For actual api interactions, `Vec<T>` is fine.
impl<T: serde::de::DeserializeOwned> FromStr for VecExt<T> {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        serde_json::from_str(string).map_err(|source| {
            // We do a map_err -> anyhow here, because we need the details inlined
            // for when the error message is shown on the cli without backtrace
            anyhow!(
                "Expected valid json for type. Got: '{}', which failed because: '{}'",
                string.to_owned(),
                source
            )
        })
    }
}
