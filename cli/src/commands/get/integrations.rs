use std::{fs::File, io::BufWriter, path::PathBuf};

use anyhow::{Context, Result};
use structopt::StructOpt;

use openapi::{
    apis::{
        configuration::Configuration,
        integrations_api::{get_all_integrations, get_integration},
    },
    models::Integration,
};

use crate::{
    printer::{print_resources_as_json, Printer},
    utils::FullName,
};

#[derive(Debug, StructOpt)]
pub struct GetIntegrationsArgs {
    #[structopt(name = "name")]
    /// The full name of the integration to get
    name: Option<FullName>,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write integrations as JSON. If not specified, stdout will be used.
    path: Option<PathBuf>,
}

/// Retrieve integrations with optional filtering and file output
pub fn get(config: &Configuration, args: &GetIntegrationsArgs, printer: &Printer) -> Result<()> {
    let GetIntegrationsArgs { name, path } = args;

    let integrations: Vec<Integration>;

    if let Some(name) = name {
        let response = get_integration(config, name.owner(), name.name())
            .context("Failed to get integration")?;
        integrations = vec![*response.integration];
    } else {
        let response = get_all_integrations(config).context("Failed to get integrations")?;
        integrations = response.integrations;
    }

    match path {
        Some(path) => {
            let file = File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?;

            print_resources_as_json(integrations, file)
        }
        None => printer.print_resources(&integrations),
    }
}
