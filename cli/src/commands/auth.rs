use anyhow::{anyhow, Result};
use structopt::StructOpt;

use reinfer_client::Client;

#[derive(Debug, StructOpt)]
pub enum AuthArgs {
    #[structopt(name = "refresh")]
    /// Refresh permissions for the current user
    RefreshPermissions {},
}

pub fn refresh_user_permissions(client: &Client, verbose: bool) -> Result<()> {
    let result = client.refresh_user_permissions()?;

    match result.permissions_refreshed {
        Some(true) => {
            if verbose {
                log::info!("Permissions Refreshed");
            }

            Ok(())
        }
        Some(_) => {
            log::error!("Failed to refresh permissions. Please login with the following link and try again\n{}", client.base_url());
            Err(anyhow!("Failed to refresh permissions"))
        }
        None => {
            if verbose {
                Err(anyhow!("Permissions refresh not relevant for user"))
            } else {
                Ok(())
            }
        }
    }
}

pub fn run(args: &AuthArgs, client: Client) -> Result<()> {
    match args {
        AuthArgs::RefreshPermissions {} => refresh_user_permissions(&client, true),
    }
}
