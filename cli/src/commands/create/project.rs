use std::{thread::sleep, time::Duration};

use crate::printer::Printer;
use crate::commands::auth::refresh_user_permissions;
use anyhow::{anyhow, Context, Result};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        projects_api::{create_project, get_all_projects},
    },
    models,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateProjectArgs {
    #[structopt(name = "project-name")]
    /// Full name of the new project
    name: String,

    #[structopt(long = "title")]
    /// Set the title of the new project
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the new project
    description: Option<String>,

    #[structopt(long = "user-ids", required = true)]
    /// The ids of users to be given initial control of the new project
    user_ids: Vec<String>,
}

pub fn create(config: &Configuration, args: &CreateProjectArgs, printer: &Printer) -> Result<()> {
    let CreateProjectArgs {
        name,
        title,
        description,
        user_ids,
    } = args;

    let project = create_project_with_wait(config, name.clone(), description.clone(), title.clone(), user_ids.clone())?;

    info!("New project `{}` created successfully", project.name);
    printer.print_resources(&[project])?;
    Ok(())
}

pub fn create_project_with_wait(
    config: &Configuration,
    name: String,
    description: Option<String>,
    title: Option<String>,
    user_ids: Vec<String>,
) -> Result<models::Project> {
    // Create the project
    let project_new = models::ProjectNew {
        title,
        description,
        user_auto_join: None,
    };

    let create_request = models::CreateProjectRequest {
        project: Box::new(project_new),
        user_ids: Some(user_ids),
        uipath_user_ids: None,
    };

    let response = create_project(config, &name, create_request)
        .context("Operation to create a project has failed")?;

    let project = *response.project;

    // Block until project is created
    let mut project_found = false;
    for _ in 0..10 {
        refresh_user_permissions(config, false)?;
        let projects_response = get_all_projects(config, None)
            .context("Failed to get projects")?;
        
        if projects_response.projects.iter().any(|p| p.name == name) {
            project_found = true;
            break;
        }
        sleep(Duration::from_secs(1));
    }

    refresh_user_permissions(config, false)?; 
    if !project_found {
        return Err(anyhow!(
            "Could not create project, timed out waiting for it to exist"
        ));
    }

    Ok(project)
}
