use anyhow::{bail, Context, Result};
use openapi::{
    apis::{
        configuration::Configuration,
        users_api::{get_user_by_id, get_users},
    },
    models::{ProjectPermission, User},
};
use structopt::StructOpt;

use crate::printer::Printer;
use crate::utils::{full_name::ProjectName, resource_identifier::UserIdentifier, get_current_user};

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

pub fn get(config: &Configuration, args: &GetUsersArgs, printer: &Printer) -> Result<()> {
    let GetUsersArgs {
        user,
        project_name_filter,
        project_permission_filter,
    } = args;

    if project_name_filter.is_none() && project_permission_filter.is_some() {
        bail!("You cannot filter on `permission` without a `project`")
    }

    let mut users = match user {
        Some(user_id) => {
            let user = match user_id {
                UserIdentifier::Id(id) => {
                    let response = get_user_by_id(config, id)
                        .context("Operation to get user has failed.")?;
                    response.user
                }
                UserIdentifier::FullName(_) => {
                    bail!("User lookup by full name is not supported. Please use user ID.")
                }
            };
            vec![user]
        }
        None => {
            let response = get_users(config)
                .context("Operation to list users has failed.")?;
            response.users
        }
    };

    if let Some(project_name) = project_name_filter {
        users.retain(|user| {
            user.organisation_permissions
                .get(project_name.as_str())
                .is_some_and(|user_permissions| {
                    if let Some(project_permission) = project_permission_filter {
                        user_permissions.contains(project_permission)
                    } else {
                        true
                    }
                })
        })
    }

    printer.print_resources(&users)
}

pub fn get_current_user_and_print(config: &Configuration, printer: &Printer) -> Result<()> {
    let user = get_current_user(config)?;
    printer.print_resources(&[user])
}
