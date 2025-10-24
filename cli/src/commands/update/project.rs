use anyhow::{Context, Result};
use log::info;
use structopt::StructOpt;

use openapi::{
    apis::{configuration::Configuration, projects_api::update_project},
    models,
};

use crate::printer::Printer;
use crate::utils::ProjectName;

#[derive(Debug, StructOpt)]
pub struct UpdateProjectArgs {
    #[structopt(name = "project-name")]
    /// Full name of the project
    name: ProjectName,

    #[structopt(long = "title")]
    /// Set the title of the project
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the project
    description: Option<String>,
}

pub fn update(config: &Configuration, args: &UpdateProjectArgs, printer: &Printer) -> Result<()> {
    let UpdateProjectArgs {
        name,
        title,
        description,
    } = args;

    let project_new = models::ProjectNew {
        title: title.clone(),
        description: description.clone(),
        user_auto_join: None,
    };

    let update_request = models::UpdateProjectRequest {
        project: Box::new(project_new),
    };

    let response = update_project(config, name.as_str(), update_request)
        .context("Operation to update a project has failed")?;

    let project = response.project;
    info!("Project `{}` updated successfully", project.name);
    printer.print_resources(&[*project])?;
    Ok(())
}
