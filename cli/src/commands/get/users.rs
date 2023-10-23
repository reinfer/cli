use anyhow::{bail, Context, Result};
use reinfer_client::{Client, ProjectName, ProjectPermission, UserIdentifier};
use structopt::StructOpt;

use crate::printer::Printer;

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

pub fn get(client: &Client, args: &GetUsersArgs, printer: &Printer) -> Result<()> {
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
            let user = client
                .get_user(user_id.clone())
                .context("Operation to get user has failed.")?;
            vec![user]
        }
        None => client
            .get_users()
            .context("Operation to list users has failed.")?,
    };

    if let Some(project_name) = project_name_filter {
        users.retain(|user| {
            user.project_permissions
                .get(project_name)
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

pub fn get_current_user(client: &Client, printer: &Printer) -> Result<()> {
    let user = client
        .get_current_user()
        .context("Operation to get the current user has failed.")?;
    printer.print_resources(&[user])
}
