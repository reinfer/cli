use anyhow::{Context, Result};
use openapi::{
    apis::{
        configuration::Configuration,
        users_api::{create_user, send_welcome_email},
    },
    models::{CreateUserRequest, GlobalPermission, ProjectPermission, UserNew, User},
};
use std::collections::hash_map::HashMap;
use structopt::StructOpt;

use crate::printer::Printer;
use crate::utils::ProjectName;

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

pub fn create(config: &Configuration, args: &CreateUserArgs, printer: &Printer) -> Result<()> {
    let CreateUserArgs {
        username,
        email,
        global_permissions,
        project,
        project_permissions_list,
        send_welcome_email,
    } = args;

    let organisation_permissions = match (project, project_permissions_list) {
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

    let user_new = UserNew {
        username: username.clone(),
        email: email.clone(),
        global_permissions: if global_permissions.is_empty() { None } else { Some(global_permissions.clone()) },
        organisation_permissions,
    };

    let request = CreateUserRequest {
        user: Box::new(user_new),
    };

    let response = create_user(config, request)
        .context("Operation to create a user has failed")?;
    
    let user = response.user;
    
    log::info!(
        "New user `{}` with email `{}` [id: {}] created successfully",
        user.username,
        user.email,
        user.id
    );

    if *send_welcome_email {
        send_welcome_email(config, &user.id)
            .context("Operation to send welcome email failed")?;
        log::info!("Welcome email sent for user '{}'", user.username);
    }

    printer.print_resources(&[user])?;
    Ok(())
}
