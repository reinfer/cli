use anyhow::{Context, Result};
use openapi::{
    apis::{
        configuration::Configuration,
        projects_api::{get_project, get_projects},
    },
    models::ProjectName,
};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetProjectsArgs {
    #[structopt(name = "project")]
    /// If specified, only list this project (name or id)
    project: Option<ProjectName>,
}

pub fn get(config: &Configuration, args: &GetProjectsArgs, printer: &Printer) -> Result<()> {
    let GetProjectsArgs { project } = args;
    let projects = if let Some(project) = project {
        vec![get_project(config, &project.0)
            .context("Operation to list projects has failed.")?
            .project]
    } else {
        let mut projects = get_projects(config)
            .context("Operation to list projects has failed.")?
            .projects;
        projects.sort_unstable_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
        projects
    };
    printer.print_resources(&projects)
}
