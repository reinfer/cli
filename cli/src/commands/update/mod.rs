mod dataset;
mod source;

use self::{dataset::UpdateDatasetArgs, source::UpdateSourceArgs};
use anyhow::Result;
use reinfer_client::Client;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum UpdateArgs {
    #[structopt(name = "source")]
    /// Update a new source
    Source(UpdateSourceArgs),

    #[structopt(name = "dataset")]
    /// Update a new dataset
    Dataset(UpdateDatasetArgs),
}

pub fn run(update_args: &UpdateArgs, client: Client) -> Result<()> {
    match update_args {
        UpdateArgs::Source(source_args) => source::update(&client, source_args),
        UpdateArgs::Dataset(dataset_args) => dataset::update(&client, dataset_args),
    }
}
