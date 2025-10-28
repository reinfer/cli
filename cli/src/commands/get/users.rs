use anyhow::{bail, Context, Result};
use structopt::StructOpt;

use openapi::{
    apis::{
        configuration::Configuration,
        users_api::{get_user_by_id, get_users},
    },
    models,
};

use crate::{
    printer::Printer,
    utils::{get_current_user, types::identifiers::UserIdentifier, ProjectName, ProjectPermission},
};

#[derive(Debug, StructOpt)]
pub struct GetUsersArgs {
    #[structopt(short = "u", long = "user")]
    /// Use to retrieve a single user with the provided id
    user: Option<UserIdentifier>,

    #[structopt(short = "o", long = "project")]
    /// Filter users by a given project
    project_name_filter: Option<ProjectName>,

    #[structopt(short = "p", long = "permission")]
    /// Filter users by a given project permission
    project_permission_filter: Option<ProjectPermission>,
}

/// Retrieve users with optional filtering by project and permissions
pub fn get(config: &Configuration, args: &GetUsersArgs, printer: &Printer) -> Result<()> {
    let GetUsersArgs {
        user,
        project_name_filter,
        project_permission_filter,
    } = args;

    validate_filter_arguments(project_name_filter, project_permission_filter)?;

    let mut users = if let Some(user_id) = user {
        vec![get_single_user(config, user_id)?]
    } else {
        get_all_users(config)?
    };

    if let Some(project_name) = project_name_filter {
        filter_users_by_project(&mut users, project_name, project_permission_filter);
    }

    printer.print_resources(&users)
}

/// Validate that filter arguments are consistent
fn validate_filter_arguments(
    project_name_filter: &Option<ProjectName>,
    project_permission_filter: &Option<ProjectPermission>,
) -> Result<()> {
    if project_name_filter.is_none() && project_permission_filter.is_some() {
        bail!("You cannot filter on `permission` without a `project`")
    }
    Ok(())
}

/// Retrieve a single user by identifier
fn get_single_user(config: &Configuration, user_id: &UserIdentifier) -> Result<models::User> {
    let user = match user_id {
        UserIdentifier::Id(id) => {
            let response = get_user_by_id(config, id)
                .with_context(|| format!("Failed to get user with ID: {id}"))?;
            *response.user
        }
        UserIdentifier::FullName(_) => {
            bail!("User lookup by full name is not supported. Please use user ID.")
        }
    };
    Ok(user)
}

/// Retrieve all users
fn get_all_users(config: &Configuration) -> Result<Vec<models::User>> {
    let response = get_users(config).context("Failed to list users")?;
    Ok(response.users)
}

/// Filter users by project and optional permission
fn filter_users_by_project(
    users: &mut Vec<models::User>,
    project_name: &ProjectName,
    project_permission_filter: &Option<ProjectPermission>,
) {
    users.retain(|user| {
        user.organisation_permissions
            .get(project_name.as_str())
            .is_some_and(|user_permissions| {
                if let Some(project_permission) = project_permission_filter {
                    let openapi_permission: models::ProjectPermission =
                        project_permission.clone().into();
                    user_permissions.contains(&openapi_permission)
                } else {
                    true
                }
            })
    });
}

pub fn get_current_user_and_print(config: &Configuration, printer: &Printer) -> Result<()> {
    let user = get_current_user(config)?;
    printer.print_resources(&[user])
}
