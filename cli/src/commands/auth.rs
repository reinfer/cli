use anyhow::{anyhow, Result};
use structopt::StructOpt;

use crate::utils::auth::refresh::refresh_user_permissions as refresh_permissions_impl;
use openapi::apis::configuration::Configuration;

#[derive(Debug, StructOpt)]
pub enum AuthArgs {
    #[structopt(name = "refresh")]
    /// Refresh permissions for the current user
    RefreshPermissions {},
}

pub fn refresh_user_permissions(config: &Configuration, verbose: bool) -> Result<()> {
    let result = refresh_permissions_impl(config)?;

    match result.permissions_refreshed {
        Some(true) => {
            if verbose {
                log::info!("Permissions Refreshed");
            }

            Ok(())
        }
        Some(_) => {
            log::error!("Failed to refresh permissions. Please login with the following link and try again\n{}", config.base_path);
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

pub fn run(args: &AuthArgs, config: &Configuration) -> Result<()> {
    match args {
        AuthArgs::RefreshPermissions {} => refresh_user_permissions(config, true),
    }
}
