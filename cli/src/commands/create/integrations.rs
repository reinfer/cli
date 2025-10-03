use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use std::path::PathBuf;

use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        integrations_api::{
            create_integration, get_integration, update_integration,
        },
    },
    models::{CreateIntegrationRequest, IntegrationNew, Integration, UpdateIntegrationRequest},
};
use structopt::StructOpt;

use crate::utils::FullName as IntegrationFullName;

#[derive(Debug, StructOpt)]
pub struct CreateIntegrationArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with integration
    path: PathBuf,

    #[structopt(name = "name")]
    /// Name of the new integration
    name: IntegrationFullName,

    #[structopt(long)]
    /// Whether to overwrite an existing integration with the same name
    overwrite: bool,
}

pub fn create(config: &Configuration, args: &CreateIntegrationArgs) -> Result<()> {
    let CreateIntegrationArgs {
        path,
        name,
        overwrite,
    } = args;

    let new_integration = read_integration(path)?;

    // Check if integration already exists
    let existing = match get_integration(config, name.owner(), name.name()) {
        Ok(response) => Some(response.integration),
        Err(_) => None, // Integration doesn't exist
    };

    if let Some(existing) = existing {
        if *overwrite {
            overwrite_integration(config, name, &new_integration, &existing)?;
            info!("Updated integration {}", name.0);
        } else {
            bail!("Provide the `--overwrite` flag to update an existing integration")
        }
    } else {
        let request = CreateIntegrationRequest {
            integration: Box::new(new_integration),
        };
        create_integration(config, name.owner(), name.name(), request)?;
        info!("Created new integration {}", name.0);
    }

    Ok(())
}

fn overwrite_integration(
    config: &Configuration,
    name: &IntegrationFullName,
    new_integration: &IntegrationNew,
    old_integration: &Integration,
) -> Result<()> {
    // Check if the updatable fields are the same
    let fields_are_same = Some(&old_integration.title) == new_integration.title.as_ref()
        && Some(Some(&old_integration.configuration)) == new_integration.configuration.as_ref().map(|c| c.as_ref())
        && old_integration.enabled == new_integration.enabled.unwrap_or(true);

    if fields_are_same {
        bail!("New integration is same as existing integration")
    }

    let old_json_str = serde_json::to_string_pretty(&old_integration)?;
    let new_json_str = serde_json::to_string_pretty(&new_integration)?;

    for diff in diff::lines(&old_json_str, &new_json_str) {
        match diff {
            diff::Result::Left(l) => println!("{}", format!("-{l}").red()),
            diff::Result::Both(l, _) => println!("{}", format!(" {l}").dimmed()),
            diff::Result::Right(r) => println!("{}", format!("+{r}").green()),
        }
    }

    if Confirm::new()
        .with_prompt(
            "Above is a summary of the changes that are about to made, do you want to continue?",
        )
        .interact()?
    {
        let request = UpdateIntegrationRequest {
            integration: Box::new(openapi::models::IntegrationUpdate {
                title: new_integration.title.clone(),
                configuration: new_integration.configuration.clone(),
                enabled: new_integration.enabled,
                updated_at: None,
            }),
        };
        update_integration(config, name.owner(), name.name(), request)?;
        Ok(())
    } else {
        bail!("Operation aborted by user")
    }
}

fn read_integration(path: &PathBuf) -> Result<IntegrationNew> {
    let integration_str = std::fs::read_to_string(path)
        .with_context(|| format!("Could not open file `{}`", path.display()))?;

    serde_json::from_str::<IntegrationNew>(&integration_str)
        .with_context(|| "Could not parse integration".to_string())
}
