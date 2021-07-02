use crate::printer::Printer;
use anyhow::{anyhow, Context, Result};
use log::info;
use reinfer_client::{
    Client, Email, GlobalPermission, NewUser, Organisation, OrganisationPermission, Username,
};
use std::collections::hash_map::HashMap;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateUserArgs {
    #[structopt(name = "username")]
    /// Username
    username: Username,

    #[structopt(name = "email")]
    /// Email
    email: Email,

    #[structopt(long = "global-permissions")]
    /// Global permissions
    global_permissions: Vec<GlobalPermission>,

    #[structopt(short = "o", long = "organisation")]
    /// Organisation
    organisation: Option<Organisation>,

    #[structopt(short = "p", long = "organisation-permissions")]
    /// Organisation permissions
    organisation_permissions_list: Vec<OrganisationPermission>,
}

pub fn create(client: &Client, args: &CreateUserArgs, printer: &Printer) -> Result<()> {
    let CreateUserArgs {
        ref username,
        ref email,
        ref global_permissions,
        ref organisation,
        ref organisation_permissions_list,
    } = *args;

    let organisation_permissions = match (organisation, organisation_permissions_list) {
        (Some(organisation), permissions) if !permissions.is_empty() => maplit::hashmap!(
            organisation.clone() => permissions.iter()
                .map(Clone::clone)
                .collect()
        ),
        (None, permissions) if permissions.is_empty() => HashMap::new(),
        _ => {
            return Err(anyhow!(
                "Arguments `--organisation` and `--organisation-permissions` have to be both specified or neither."
            ));
        }
    };

    let user = client
        .create_user(NewUser {
            username,
            email,
            global_permissions,
            organisation_permissions: &organisation_permissions,
        })
        .context("Operation to create a user has failed.")?;
    info!(
        "New user `{}` with email `{}` [id: {}] created successfully",
        user.username.0, user.email.0, user.id.0
    );
    printer.print_resources(&[user])?;
    Ok(())
}
