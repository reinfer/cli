//! Commands for creating and managing integrations
//!
//! This module provides functionality to:
//! - Create new integrations from JSON configuration files
//! - Update existing integrations with change confirmation
//! - Display detailed diffs before applying changes
//! - Handle integration validation and error reporting

// Standard library imports
use std::path::PathBuf;

// External crate imports
use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use log::info;
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{
        configuration::Configuration,
        integrations_api::{create_integration, get_integration, update_integration},
    },
    models::{CreateIntegrationRequest, Integration, IntegrationNew, UpdateIntegrationRequest},
};

// Local crate imports
use crate::utils::FullName as IntegrationFullName;

/// Command line arguments for creating or updating integrations
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

/// Create a new integration or update an existing one from a JSON configuration file
///
/// This function handles:
/// - Reading integration configuration from JSON files
/// - Checking for existing integrations with the same name
/// - Displaying change diffs for updates
/// - User confirmation for destructive operations
pub fn create(config: &Configuration, args: &CreateIntegrationArgs) -> Result<()> {
    let CreateIntegrationArgs {
        path,
        name,
        overwrite,
    } = args;

    let new_integration = read_integration_from_file(path)?;

    let existing = check_existing_integration(config, name)?;

    match existing {
        Some(existing_integration) => {
            if *overwrite {
                update_existing_integration(config, name, &new_integration, &existing_integration)?;
                info!("Updated integration {}", name.0);
            } else {
                bail!("Provide the `--overwrite` flag to update an existing integration")
            }
        }
        None => {
            create_new_integration(config, name, new_integration)?;
            info!("Created new integration {}", name.0);
        }
    }

    Ok(())
}

/// Update an existing integration with user confirmation after showing diff
fn update_existing_integration(
    config: &Configuration,
    name: &IntegrationFullName,
    new_integration: &IntegrationNew,
    old_integration: &Integration,
) -> Result<()> {
    if are_integrations_identical(new_integration, old_integration) {
        bail!("New integration is same as existing integration")
    }

    display_integration_diff(old_integration, new_integration)?;

    if confirm_integration_update()? {
        perform_integration_update(config, name, new_integration)?;
        Ok(())
    } else {
        bail!("Operation aborted by user")
    }
}

/// Read and parse integration configuration from a JSON file
fn read_integration_from_file(path: &PathBuf) -> Result<IntegrationNew> {
    let integration_str = std::fs::read_to_string(path)
        .with_context(|| format!("Could not open file `{}`", path.display()))?;

    serde_json::from_str::<IntegrationNew>(&integration_str)
        .with_context(|| format!("Could not parse integration from file `{}`", path.display()))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if an integration with the given name already exists
fn check_existing_integration(
    config: &Configuration,
    name: &IntegrationFullName,
) -> Result<Option<Integration>> {
    match get_integration(config, name.owner(), name.name()) {
        Ok(response) => Ok(Some(*response.integration)),
        Err(_) => Ok(None), // Integration doesn't exist
    }
}

/// Create a new integration from the provided configuration
fn create_new_integration(
    config: &Configuration,
    name: &IntegrationFullName,
    integration: IntegrationNew,
) -> Result<()> {
    let request = CreateIntegrationRequest {
        integration: Box::new(integration),
    };
    create_integration(config, name.owner(), name.name(), request)
        .context("Failed to create integration")?;
    Ok(())
}

/// Check if two integrations are functionally identical
fn are_integrations_identical(
    new_integration: &IntegrationNew,
    old_integration: &Integration,
) -> bool {
    Some(&old_integration.title) == new_integration.title.as_ref()
        && Some(Some(&old_integration.configuration))
            == new_integration.configuration.as_ref().map(|c| c.as_ref())
        && old_integration.enabled == new_integration.enabled.unwrap_or(true)
}

/// Display a colored diff between old and new integration configurations
fn display_integration_diff(
    old_integration: &Integration,
    new_integration: &IntegrationNew,
) -> Result<()> {
    let old_json_str = serde_json::to_string_pretty(&old_integration)
        .context("Failed to serialize old integration")?;
    let new_json_str = serde_json::to_string_pretty(&new_integration)
        .context("Failed to serialize new integration")?;

    for diff in diff::lines(&old_json_str, &new_json_str) {
        match diff {
            diff::Result::Left(l) => println!("{}", format!("-{l}").red()),
            diff::Result::Both(l, _) => println!("{}", format!(" {l}").dimmed()),
            diff::Result::Right(r) => println!("{}", format!("+{r}").green()),
        }
    }
    Ok(())
}

/// Prompt user for confirmation to proceed with integration update
fn confirm_integration_update() -> Result<bool> {
    Confirm::new()
        .with_prompt(
            "Above is a summary of the changes that are about to be made, do you want to continue?",
        )
        .interact()
        .context("Failed to get user confirmation")
}

/// Perform the actual integration update via API call
fn perform_integration_update(
    config: &Configuration,
    name: &IntegrationFullName,
    new_integration: &IntegrationNew,
) -> Result<()> {
    let request = UpdateIntegrationRequest {
        integration: Box::new(openapi::models::IntegrationUpdate {
            title: new_integration.title.clone(),
            configuration: new_integration.configuration.clone(),
            enabled: new_integration.enabled,
            updated_at: None,
        }),
    };
    update_integration(config, name.owner(), name.name(), request)
        .context("Failed to update integration")?;
    Ok(())
}
