use std::{thread::sleep, time::Duration};

use crate::{commands::auth::refresh_user_permissions, printer::Printer};
use anyhow::{anyhow, Context, Result};
use log::info;
use reinfer_client::{Client, NewProject, Project, ProjectName, UserId};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateProjectArgs {
    #[structopt(name = "project-name")]
    /// Full name of the new project
    name: ProjectName,

    #[structopt(long = "title")]
    /// Set the title of the new project
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the new project
    description: Option<String>,

    #[structopt(long = "user-ids", required = true)]
    /// The ids of users to be given initial control of the new project
    user_ids: Vec<UserId>,
}

pub fn create(client: &Client, args: &CreateProjectArgs, printer: &Printer) -> Result<()> {
    let CreateProjectArgs {
        name,
        title,
        description,
        user_ids,
    } = args;

    let project = create_project(client, name, description, title, user_ids)?;

    info!("New project `{}` created successfully", project.name.0,);
    printer.print_resources(&[project])?;
    Ok(())
}

pub fn create_project(
    client: &Client,
    name: &ProjectName,
    description: &Option<String>,
    title: &Option<String>,
    user_ids: &Vec<UserId>,
) -> Result<Project> {
    let project = client
        .create_project(
            name,
            NewProject {
                title: title.as_deref(),
                description: description.as_deref(),
            },
            user_ids,
        )
        .context("Operation to create a project has failed")?;

    // Block until project is created
    let mut project_found = false;
    for _ in 0..10 {
        refresh_user_permissions(client, false)?;
        let projects = client.get_projects()?;
        if projects.iter().any(|p| p.name.0 == name.0) {
            project_found = true;
            break;
        }
        sleep(Duration::from_secs(1));
    }

    refresh_user_permissions(client, false)?;

    if !project_found {
        return Err(anyhow!(
            "Could not create project, timed out waiting for it to exist"
        ));
    }

    Ok(project)
}
