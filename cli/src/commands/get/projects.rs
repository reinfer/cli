use anyhow::{Context, Result};
use structopt::StructOpt;

use openapi::apis::{
    configuration::Configuration,
    projects_api::{get_all_projects, get_project},
};

use crate::{printer::Printer, utils::ProjectName};

#[derive(Debug, StructOpt)]
pub struct GetProjectsArgs {
    #[structopt(name = "project")]
    /// If specified, only list this project (name or id)
    project: Option<ProjectName>,
}

/// Retrieve projects with optional filtering by project name
pub fn get(config: &Configuration, args: &GetProjectsArgs, printer: &Printer) -> Result<()> {
    let GetProjectsArgs { project } = args;
    let projects = if let Some(project) = project {
        vec![
            *get_project(config, &project.0)
                .context("Operation to list projects has failed.")?
                .project,
        ]
    } else {
        let mut projects = get_all_projects(config, None)
            .context("Operation to list projects has failed.")?
            .projects;
        projects.sort_unstable_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
        projects
    };
    printer.print_resources(&projects)
}
