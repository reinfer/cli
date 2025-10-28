//! Commands for creating users with permissions and project associations
//!
//! This module provides functionality to:
//! - Create users with global permissions
//! - Associate users with projects and project-specific permissions
//! - Send welcome emails to new users
//! - Validate permission combinations and requirements

// Standard library imports
use std::collections::HashMap;

// External crate imports
use anyhow::{Context, Result};
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{configuration::Configuration, users_api::create_user},
    models::{CreateUserRequest, UserNew},
};

// Local crate imports
use crate::{
    printer::Printer,
    utils::{GlobalPermission, ProjectName, ProjectPermission},
};

/// Command line arguments for creating a user with permissions
#[derive(Debug, StructOpt)]
pub struct CreateUserArgs {
    #[structopt(name = "username")]
    /// Username for the new user
    username: String,

    #[structopt(name = "email")]
    /// Email address of the new user
    email: String,

    #[structopt(long = "global-permissions")]
    /// Global permissions to give to the new user
    global_permissions: Vec<GlobalPermission>,

    #[structopt(short = "p", long = "project")]
    /// Add the user to this project with the permissions provided with --project-permissions
    project: Option<ProjectName>,

    #[structopt(long = "project-permissions")]
    /// Project permissions, required if --project is used
    project_permissions_list: Vec<ProjectPermission>,

    #[structopt(short = "w", long = "send-welcome-email")]
    /// Send the user a welcome email
    send_welcome_email: bool,
}

/// Create a new user with specified permissions and optional project association
///
/// This function handles:
/// - Permission validation and organization
/// - User creation via API
/// - Optional welcome email sending
/// - Resource printing for confirmation
pub fn create(config: &Configuration, args: &CreateUserArgs, printer: &Printer) -> Result<()> {
    let CreateUserArgs {
        username,
        email,
        global_permissions,
        project,
        project_permissions_list,
        send_welcome_email,
    } = args;

    let organisation_permissions =
        build_organisation_permissions(project, project_permissions_list)?;

    let user_new = build_user_new(
        username.clone(),
        email.clone(),
        global_permissions,
        organisation_permissions,
    );

    let request = CreateUserRequest {
        user: Box::new(user_new),
    };

    let user = create_user_via_api(config, request)?;

    log::info!(
        "New user `{}` with email `{}` [id: {}] created successfully",
        user.username,
        user.email,
        user.id
    );

    if *send_welcome_email {
        send_user_welcome_email(config, &user)?;
    }

    printer.print_resources(&[user])?;
    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Build organization permissions map from project and permissions arguments
fn build_organisation_permissions(
    project: &Option<ProjectName>,
    project_permissions_list: &[ProjectPermission],
) -> Result<HashMap<String, Vec<openapi::models::ProjectPermission>>> {
    match (project, project_permissions_list) {
        (Some(project), permissions) if !permissions.is_empty() => {
            let mut map = HashMap::new();
            map.insert(
                project.0.clone(),
                permissions.iter().map(|p| p.0.clone()).collect(),
            );
            Ok(map)
        }
        (None, permissions) if permissions.is_empty() => Ok(HashMap::new()),
        _ => {
            anyhow::bail!(
                "Arguments `--project` and `--project-permissions` have to be both specified or neither"
            )
        }
    }
}

/// Build a UserNew object with the specified parameters
fn build_user_new(
    username: String,
    email: String,
    global_permissions: &[GlobalPermission],
    organisation_permissions: HashMap<String, Vec<openapi::models::ProjectPermission>>,
) -> UserNew {
    UserNew {
        username,
        email,
        global_permissions: if global_permissions.is_empty() {
            None
        } else {
            Some(global_permissions.iter().map(|p| p.0.clone()).collect())
        },
        organisation_permissions: if organisation_permissions.is_empty() {
            None
        } else {
            Some(organisation_permissions)
        },
    }
}

/// Create a user via API call and return the created user object
fn create_user_via_api(
    config: &Configuration,
    request: CreateUserRequest,
) -> Result<openapi::models::User> {
    let response = create_user(config, request).context("Operation to create a user has failed")?;
    Ok(*response.user)
}

/// Send welcome email to a newly created user
fn send_user_welcome_email(config: &Configuration, user: &openapi::models::User) -> Result<()> {
    openapi::apis::users_api::send_welcome_email(config, &user.id)
        .context("Operation to send welcome email failed")?;
    log::info!("Welcome email sent for user");
    Ok(())
}
