mod dataset;
mod project;
mod source;

use self::{dataset::UpdateDatasetArgs, project::UpdateProjectArgs, source::UpdateSourceArgs};
use crate::printer::Printer;
use anyhow::Result;
use reinfer_client::Client;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum UpdateArgs {
    #[structopt(name = "source")]
    /// Update an existing source
    Source(UpdateSourceArgs),

    #[structopt(name = "dataset")]
    /// Update an existing dataset
    Dataset(UpdateDatasetArgs),

    #[structopt(name = "project")]
    /// Update an existing project
    Project(UpdateProjectArgs),
}

pub fn run(update_args: &UpdateArgs, client: Client, printer: &Printer) -> Result<()> {
    match update_args {
        UpdateArgs::Source(source_args) => source::update(&client, source_args, printer),
        UpdateArgs::Dataset(dataset_args) => dataset::update(&client, dataset_args, printer),
        UpdateArgs::Project(project_args) => project::update(&client, project_args, printer),
    }
}
