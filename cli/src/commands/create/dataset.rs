use crate::printer::Printer;
use anyhow::{anyhow, Context, Error, Result};
use log::info;
use reinfer_client::{
    Client, DatasetFullName, NewDataset, NewEntityDef, NewLabelDef, SourceIdentifier,
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

    #[structopt(long = "has-sentiment")]
    /// Enable sentiment prediction for the dataset
    has_sentiment: Option<bool>,

    #[structopt(short = "s", long = "source")]
    /// Names or ids of the sources in the dataset
    sources: Vec<SourceIdentifier>,

    #[structopt(short = "e", long = "entity-defs", default_value = "[]")]
    /// Entity defs to create at dataset creation, as json
    entity_defs: VecExt<NewEntityDef>,

    #[structopt(long = "label-defs", default_value = "[]")]
    /// Label defs to create at dataset creation, as json
    label_defs: VecExt<NewLabelDef>,

    #[structopt(long = "model-family")]
    /// Model family to use for the new dataset
    model_family: Option<String>,

    /// Dataset ID of the dataset to copy annotations from
    #[structopt(long = "copy-annotations-from")]
    copy_annotations_from: Option<String>,
}

pub fn create(client: &Client, args: &CreateDatasetArgs, printer: &Printer) -> Result<()> {
    let CreateDatasetArgs {
        ref name,
        ref title,
        ref description,
        ref has_sentiment,
        ref sources,
        ref entity_defs,
        ref label_defs,
        ref model_family,
        ref copy_annotations_from,
    } = *args;

    let source_ids = {
        let mut source_ids = Vec::with_capacity(sources.len());
        for source in sources.iter() {
            source_ids.push(
                client
                    .get_source(source.clone())
                    .context("Operation to get source has failed")?
                    .id,
            );
        }
        source_ids
    };

    // Unwrap the inner values, we only need the outer for argument parsing
    let entity_defs = &entity_defs.0;
    let label_defs = &label_defs.0;
    let dataset = client
        .create_dataset(
            name,
            NewDataset {
                source_ids: &source_ids,
                title: title.as_deref(),
                description: description.as_deref(),
                has_sentiment: *has_sentiment,
                entity_defs: if entity_defs.is_empty() {
                    None
                } else {
                    Some(entity_defs)
                },
                label_defs: if label_defs.is_empty() {
                    None
                } else {
                    Some(&label_defs[..])
                },
                model_family: model_family.as_deref(),
                copy_annotations_from: copy_annotations_from.as_deref(),
            },
        )
        .context("Operation to create a dataset has failed.")?;
    info!(
        "New dataset `{}` [id: {}] created successfully",
        dataset.full_name().0,
        dataset.id.0,
    );
    printer.print_resources(&[dataset])?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct VecExt<T>(pub Vec<T>);

/// Utility type for foreign trait interactions.
///
/// For actual api interatctions, `Vec<T>` is fine.
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
