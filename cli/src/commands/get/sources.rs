use anyhow::{Context, Result};
use reinfer_client::{Client, SourceIdentifier};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetSourcesArgs {
    #[structopt(name = "source")]
    /// If specified, only list this source (name or id)
    source: Option<SourceIdentifier>,
}

pub fn get(client: &Client, args: &GetSourcesArgs, printer: &Printer) -> Result<()> {
    let GetSourcesArgs { source } = args;
    let sources = if let Some(source) = source {
        vec![client
            .get_source(source.clone())
            .context("Operation to list sources has failed.")?]
    } else {
        let mut sources = client
            .get_sources()
            .context("Operation to list sources has failed.")?;
        sources.sort_unstable_by(|lhs, rhs| {
            (&lhs.owner.0, &lhs.name.0).cmp(&(&rhs.owner.0, &rhs.name.0))
        });
        sources
    };
    printer.print_resources(&sources)
}
