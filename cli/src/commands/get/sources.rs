use anyhow::{Context, Result};
use reinfer_client::{Client, SourceIdentifier};
use std::collections::HashMap;
use structopt::StructOpt;

use crate::printer::{PrintableSource, Printer};

#[derive(Debug, StructOpt)]
pub struct GetSourcesArgs {
    #[structopt(name = "source")]
    /// If specified, only list this source (name or id)
    source: Option<SourceIdentifier>,
}

pub async fn get(client: &Client, args: &GetSourcesArgs, printer: &Printer) -> Result<()> {
    let GetSourcesArgs { source } = args;
    let sources = if let Some(source) = source {
        vec![client
            .get_source(source.clone())
            .await
            .context("Operation to list sources has failed.")?]
    } else {
        let mut sources = client
            .get_sources()
            .await
            .context("Operation to list sources has failed.")?;
        sources.sort_unstable_by(|lhs, rhs| {
            (&lhs.owner.0, &lhs.name.0).cmp(&(&rhs.owner.0, &rhs.name.0))
        });
        sources
    };

    let buckets: HashMap<_, _> = client
        .get_buckets()
        .await
        .context("Operation to list buckets has failed.")?
        .into_iter()
        .map(|bucket| (bucket.id.clone(), bucket))
        .collect();

    let printable_sources: Vec<PrintableSource> = sources
        .into_iter()
        .map(|source| {
            let bucket = source
                .bucket_id
                .as_ref()
                .and_then(|id| buckets.get(id))
                .cloned();
            PrintableSource { source, bucket }
        })
        .collect();

    printer.print_resources(&printable_sources)
}
