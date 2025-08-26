use anyhow::{bail, Context, Result};
use log::info;
use structopt::StructOpt;

use openapi::{
    apis::{configuration::Configuration, datasets_api::update_dataset},
    models::{DatasetUpdate, UpdateDatasetRequest},
};

use crate::printer::Printer;
use crate::utils::types::identifiers::{resolve_source, DatasetIdentifier, SourceIdentifier};

/// Update a dataset.
#[derive(Debug, StructOpt)]
pub struct UpdateDatasetArgs {
    #[structopt(name = "dataset")]
    /// Name or id of the dataset to delete
    dataset: DatasetIdentifier,

    #[structopt(long = "title")]
    /// Set the title of the dataset
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the dataset
    description: Option<String>,

    #[structopt(short = "s", long = "source")]
    /// Names or ids of the sources in the dataset
    sources: Option<Vec<SourceIdentifier>>,
}

pub fn update(config: &Configuration, args: &UpdateDatasetArgs, printer: &Printer) -> Result<()> {
    let UpdateDatasetArgs {
        dataset,
        title,
        description,
        sources,
    } = args;

    // Get source IDs if sources are provided
    let source_ids = sources
        .as_ref()
        .map::<Result<Vec<String>>, _>(|sources| {
            sources
                .iter()
                .map(|source| {
                    let resolved_source = resolve_source(config, source)?;
                    Ok(resolved_source.id)
                })
                .collect()
        })
        .transpose()
        .context("Operation to get sources failed")?;

    // Get dataset full name
    let (owner, dataset_name) = match dataset {
        DatasetIdentifier::FullName(name) => (name.owner().to_string(), name.name().to_string()),
        DatasetIdentifier::Id(_) => {
            bail!("Dataset lookup by ID is not supported. Please use the full name format 'owner/dataset'");
        }
    };

    // Create the dataset update request
    let dataset_update = DatasetUpdate {
        title: title.clone(),
        description: description.clone(),
        source_ids,
        entity_kinds: None,
        entity_defs: None,
        general_fields: None,
        _label_properties: None,
        debug_config_json: None,
        model_family: None,
        _timezone: None,
        _preferred_locales: None,
        _dataset_flags: None,
        _model_config: None,
        _default_label_group_instructions: None,
    };

    let request = UpdateDatasetRequest {
        dataset: Box::new(dataset_update),
    };

    // Update the dataset
    let response = update_dataset(config, &owner, &dataset_name, request)
        .context("Operation to update a dataset has failed.")?;

    let dataset = response.dataset;
    info!(
        "Dataset `{}` [id: {}] updated successfully",
        format!("{}/{}", dataset.owner, dataset.name),
        dataset.id,
    );
    printer.print_resources(&[*dataset])?;
    Ok(())
}
