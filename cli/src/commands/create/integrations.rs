use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use std::path::PathBuf;

use log::info;
use reinfer_client::{
    resources::integration::{Integration, NewIntegration},
    Client, IntegrationFullName,
};
use structopt::StructOpt;

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

pub fn create(client: &Client, args: &CreateIntegrationArgs) -> Result<()> {
    let CreateIntegrationArgs {
        path,
        name,
        overwrite,
    } = args;

    let new_integration = read_integration(path)?;

    let mut integrations = client.get_integrations()?;

    integrations
        .retain(|integration| format!("{}/{}", integration.owner.0, integration.name.0) == name.0);

    if let Some(existing) = integrations.first() {
        if *overwrite {
            overwrite_integration(client, name, &new_integration, existing)?;
            info!("Updated integration {}", name.0);
        } else {
            bail!("Provide the `--overwrite` flag to update an existing integration")
        }
    } else {
        client.put_integration(name, &new_integration)?;
        info!("Created new integration {}", name.0);
    }

    Ok(())
}

fn overwrite_integration(
    client: &Client,
    name: &IntegrationFullName,
    new_integration: &NewIntegration,
    old_integration: &Integration,
) -> Result<()> {
    let old_integration: NewIntegration =
        serde_json::from_str(&serde_json::to_string(old_integration)?)?;

    if *new_integration == old_integration {
        bail!("New integration is same as existing integration")
    }

    let old_json_str = serde_json::to_string_pretty(&old_integration)?;
    let new_json_str = serde_json::to_string_pretty(&new_integration)?;

    for diff in diff::lines(&old_json_str, &new_json_str) {
        match diff {
            diff::Result::Left(l) => println!("{}", format!("-{}", l).red()),
            diff::Result::Both(l, _) => println!("{}", format!(" {}", l).dimmed()),
            diff::Result::Right(r) => println!("{}", format!("+{}", r).green()),
        }
    }

    if Confirm::new()
        .with_prompt(
            "Above is a summary of the changes that are about to made, do you want to continue?",
        )
        .interact()?
    {
        client.post_integration(name, new_integration)?;
        Ok(())
    } else {
        bail!("Operation aborted by user")
    }
}

fn read_integration(path: &PathBuf) -> Result<NewIntegration> {
    let integration_str = std::fs::read_to_string(path)
        .with_context(|| format!("Could not open file `{}`", path.display()))?;

    serde_json::from_str::<NewIntegration>(&integration_str)
        .with_context(|| "Could not parse integration".to_string())
}
