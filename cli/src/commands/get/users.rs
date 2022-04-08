use anyhow::{Context, Result};
use reinfer_client::{Client, UserIdentifier};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetUsersArgs {
    #[structopt(short = "u", long = "user")]
    /// Use to retrieve a single user with the provided id
    user: Option<UserIdentifier>,
}

pub fn get(client: &Client, args: &GetUsersArgs, printer: &Printer) -> Result<()> {
    let GetUsersArgs { user } = args;
    match user {
        Some(user_id) => {
            let user = client
                .get_user(user_id.clone())
                .context("Operation to get user has failed.")?;
            printer.print_resources(&[user])
        }
        None => {
            let users = client
                .get_users()
                .context("Operation to list users has failed.")?;
            printer.print_resources(&users)
        }
    }
}

pub fn get_current_user(client: &Client, printer: &Printer) -> Result<()> {
    let user = client
        .get_current_user()
        .context("Operation to get the current user has failed.")?;
    printer.print_resources(&[user])
}
