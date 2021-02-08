use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    resources::comment::EntityKind, Client, DatasetFullName, NewDataset, SourceIdentifier,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateDatasetArgs {
    #[structopt(name = "dataset-name")]
    /// Full name of the new dataset <owner>/<name>
    name: String,

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

    #[structopt(short = "e", long = "entity-kinds")]
    /// Entity kinds to enable at dataset creation (pass one flag for each kind)
    entity_kinds: Vec<EntityKind>,
}

pub fn create(client: &Client, args: &CreateDatasetArgs) -> Result<()> {
    let CreateDatasetArgs {
        ref name,
        ref title,
        ref description,
        ref has_sentiment,
        ref sources,
        ref entity_kinds,
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
            &DatasetFullName(name.clone()),
            NewDataset {
                source_ids: &source_ids,
                title: title.as_ref().map(|title| title.as_str()),
                description: description.as_ref().map(|description| description.as_str()),
                has_sentiment: *has_sentiment,
                entity_kinds: &entity_kinds,
            },
        )
        .context("Operation to create a dataset has failed.")?;
    info!(
        "New dataset `{}` [id: {}] created successfully",
        dataset.full_name().0,
        dataset.id.0,
    );

    Ok(())
}
