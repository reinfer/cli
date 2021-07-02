use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    resources::dataset::EntityDefs, Client, DatasetFullName, NewDataset, SourceIdentifier,
};
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
    entity_defs: EntityDefs,

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
