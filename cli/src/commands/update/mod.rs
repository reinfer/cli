mod dataset;
mod project;
mod source;
mod users;

use self::{
    dataset::UpdateDatasetArgs, project::UpdateProjectArgs, source::UpdateSourceArgs,
    users::UpdateUsersArgs,
};
use crate::printer::Printer;
use anyhow::Result;
use openapi::apis::configuration::Configuration;
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

    #[structopt(name = "users")]
    /// Update existing users
    Users(UpdateUsersArgs),
}

pub fn run(update_args: &UpdateArgs, config: &Configuration, printer: &Printer) -> Result<()> {
    match update_args {
        UpdateArgs::Source(source_args) => source::update(config, source_args, printer),
        UpdateArgs::Dataset(dataset_args) => dataset::update(config, dataset_args, printer),
        UpdateArgs::Project(project_args) => project::update(config, project_args, printer),
        UpdateArgs::Users(users_args) => users::update(config, users_args),
    }
}
