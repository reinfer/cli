use anyhow::{Context, Result};
use reinfer_client::{Client, ProjectName};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetProjectsArgs {
    #[structopt(name = "project")]
    /// If specified, only list this project (name or id)
    project: Option<ProjectName>,
}

pub async fn get(client: &Client, args: &GetProjectsArgs, printer: &Printer) -> Result<()> {
    let GetProjectsArgs { project } = args;
    let projects = if let Some(project) = project {
        vec![client
            .get_project(project)
            .await
            .context("Operation to list projects has failed.")?]
    } else {
        let mut projects = client
            .get_projects()
            .await
            .context("Operation to list projects has failed.")?;
        projects.sort_unstable_by(|lhs, rhs| lhs.name.0.cmp(&rhs.name.0));
        projects
    };
    printer.print_resources(&projects)
}
