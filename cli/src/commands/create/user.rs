use anyhow::{Context, Result};
use reinfer_client::{
    Client, GlobalPermission, NewUser, ProjectName, ProjectPermission, UserEmail, Username,
};
use std::collections::hash_map::HashMap;
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct CreateUserArgs {
    #[structopt(name = "username")]
    /// Username for the new user
    username: Username,

    #[structopt(name = "email")]
    /// Email address of the new user
    email: UserEmail,

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

pub fn create(client: &Client, args: &CreateUserArgs, printer: &Printer) -> Result<()> {
    let CreateUserArgs {
        username,
        email,
        global_permissions,
        project,
        project_permissions_list,
        send_welcome_email,
    } = args;

    let project_permissions = match (project, project_permissions_list) {
        (Some(project), permissions) if !permissions.is_empty() => maplit::hashmap!(
            project.clone() => permissions.iter().cloned().collect()
        ),
        (None, permissions) if permissions.is_empty() => HashMap::new(),
        _ => {
            anyhow::bail!(
                "Arguments `--project` and `--project-permissions` have to be both specified or neither"
            );
        }
    };

    let user = client
        .create_user(NewUser {
            username,
            email,
            global_permissions,
            project_permissions: &project_permissions,
        })
        .context("Operation to create a user has failed")?;
    log::info!(
        "New user `{}` with email `{}` [id: {}] created successfully",
        user.username.0,
        user.email.0,
        user.id.0
    );

    if *send_welcome_email {
        client
            .send_welcome_email(user.id.clone())
            .context("Operation to send welcome email failed")?;
        log::info!("Welcome email sent for user '{}'", user.username.0);
    }

    printer.print_resources(&[user])?;
    Ok(())
}
