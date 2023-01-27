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
use reinfer_client::Client;
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
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

pub async fn run(update_args: UpdateArgs, client: Client, printer: &Printer) -> Result<()> {
    match update_args {
        UpdateArgs::Source(source_args) => source::update(&client, source_args, printer).await,
        UpdateArgs::Dataset(dataset_args) => dataset::update(&client, dataset_args, printer).await,
        UpdateArgs::Project(project_args) => project::update(&client, project_args, printer).await,
        UpdateArgs::Users(users_args) => users::update(&client, &users_args).await,
    }
}
