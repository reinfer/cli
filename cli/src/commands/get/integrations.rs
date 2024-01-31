use std::{fs::File, io::BufWriter, path::PathBuf};

use anyhow::{Context, Result};
use reinfer_client::{resources::integration::Integration, Client, IntegrationFullName};
use structopt::StructOpt;

use crate::printer::{print_resources_as_json, Printer};

#[derive(Debug, StructOpt)]
pub struct GetIntegrationsArgs {
    #[structopt(name = "name")]
    /// The full name of the integration to get
    name: Option<IntegrationFullName>,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write integrations as JSON. If not specified, stdout will be used.
    path: Option<PathBuf>,
}

pub fn get(client: &Client, args: &GetIntegrationsArgs, printer: &Printer) -> Result<()> {
    let GetIntegrationsArgs { name, path } = args;

    let integrations: Vec<Integration>;

    if let Some(name) = name {
        integrations = vec![client.get_integration(name)?];
    } else {
        integrations = client.get_integrations()?;
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
