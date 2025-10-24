//! Commands for creating projects with user permissions
//!
//! This module provides functionality to:
//! - Create new projects with initial user control
//! - Handle project creation with retry logic
//! - Wait for project availability after creation
//! - Refresh user permissions automatically

// Standard library imports
use std::{thread::sleep, time::Duration};

// External crate imports
use anyhow::{anyhow, Context, Result};
use log::info;
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{
        configuration::Configuration,
        projects_api::{create_project, get_all_projects},
    },
    models,
};

// Local crate imports
use crate::{commands::auth::refresh_user_permissions, printer::Printer};

/// Command line arguments for creating a project
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

/// Create a new project and wait for it to become available
///
/// This function handles the complete project creation workflow including
/// waiting for the project to be properly initialized and accessible
pub fn create(config: &Configuration, args: &CreateProjectArgs, printer: &Printer) -> Result<()> {
    let CreateProjectArgs {
        name,
        title,
        description,
        user_ids,
    } = args;

    let project = create_project_with_wait(
        config,
        name.clone(),
        description.clone(),
        title.clone(),
        user_ids.clone(),
    )?;

    info!("New project `{}` created successfully", project.name);
    printer.print_resources(&[project])?;
    Ok(())
}

/// Create a project and wait for it to become available with permission refresh
///
/// This utility function is also used by other modules that need to create projects
/// as part of larger workflows. It includes built-in retry logic and permission refresh.
pub fn create_project_with_wait(
    config: &Configuration,
    name: String,
    description: Option<String>,
    title: Option<String>,
    user_ids: Vec<String>,
) -> Result<models::Project> {
    let project = perform_project_creation(config, &name, description, title, user_ids)?;

    wait_for_project_availability(config, &name)?;

    Ok(project)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Perform the actual project creation via API call
fn perform_project_creation(
    config: &Configuration,
    name: &str,
    description: Option<String>,
    title: Option<String>,
    user_ids: Vec<String>,
) -> Result<models::Project> {
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

    let response = create_project(config, name, create_request)
        .context("Operation to create a project has failed")?;

    Ok(*response.project)
}

/// Wait for project to become available with retry logic
fn wait_for_project_availability(config: &Configuration, name: &str) -> Result<()> {
    const MAX_RETRIES: u32 = 10;
    const RETRY_DELAY: Duration = Duration::from_secs(1);

    let mut project_found = false;
    for attempt in 0..MAX_RETRIES {
        refresh_user_permissions(config, false).context("Failed to refresh user permissions")?;

        let projects_response = get_all_projects(config, None).context("Failed to get projects")?;

        if projects_response.projects.iter().any(|p| p.name == name) {
            project_found = true;
            break;
        }

        if attempt < MAX_RETRIES - 1 {
            sleep(RETRY_DELAY);
        }
    }

    // Final permission refresh
    refresh_user_permissions(config, false)
        .context("Failed to refresh user permissions after project creation")?;

    if !project_found {
        return Err(anyhow!(
            "Could not create project, timed out waiting for it to exist after {} attempts",
            MAX_RETRIES
        ));
    }

    Ok(())
}
