use anyhow::{anyhow, Context, Error, Result};

use colored::Colorize;
use reinfer_client::{
    BucketIdentifier, Client,
};
use serde::Deserialize;
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

use crate::{
    printer::print_resources_as_json,
    progress::{Options as ProgressOptions, Progress},
};

#[derive(Debug, StructOpt)]
pub struct GetManyEmailsArgs {
    #[structopt(name = "bucket")]
    /// Source name or id
    bucket: BucketIdentifier,

    #[structopt(long)]
    /// Don't display a progress bar (only applicable when --file is used).
    no_progress: bool,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write comments as JSON. If not specified, stdout will be used.
    path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct StructExt<T>(pub T);

impl<T: serde::de::DeserializeOwned> FromStr for StructExt<T> {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        serde_json::from_str(string).map_err(|source| {
            anyhow!(
                "Expected valid json for type. Got: '{}', which failed because: '{}'",
                string.to_owned(),
                source
            )
        })
    }
}

pub fn get_many(client: &Client, args: &GetManyEmailsArgs) -> Result<()> {
    let GetManyEmailsArgs {
        bucket,
        no_progress,
        path,
    } = args;

    let file = match path {
        Some(path) => Some(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?,
        ),
        None => None,
    };

    if let Some(file) = file {
        download_emails(client, bucket.clone(), file)
    } else {
        download_emails(
            client,
            bucket.clone(),
            io::stdout().lock(),
        )
    }
}


fn download_emails(
    client: &Client,
    bucket_identifier: BucketIdentifier,
    mut writer: impl Write,
) -> Result<()> {
    let bucket = client
        .get_bucket(bucket_identifier)
        .context("Operation to get source has failed.")?;
    let statistics = Arc::new(Statistics::new());

    client
        .get_emails_iter(&bucket.full_name(), None)
        .try_for_each(|page| {
            let page = page.context("Operation to get emails has failed.")?;
            statistics.add_emails(page.len());
            print_resources_as_json(
                page.into_iter(),
                &mut writer,
            )
        })?;
    log::info!(
        "Successfully downloaded {} emails.",
        statistics.num_downloaded(),
    );
    Ok(())
}

const DEFAULT_QUERY_PAGE_SIZE: usize = 128;

#[derive(Debug)]
pub struct Statistics {
    downloaded: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            downloaded: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_emails(&self, num_downloaded: usize) {
        self.downloaded.fetch_add(num_downloaded, Ordering::SeqCst);
    }

    #[inline]
    fn num_downloaded(&self) -> usize {
        self.downloaded.load(Ordering::SeqCst)
    }

}

fn get_emails_progress_bar(
    total_bytes: u64,
    statistics: &Arc<Statistics>,
    show_annotated: bool,
) -> Progress {
    Progress::new(
        move |statistics| {
            let num_downloaded = statistics.num_downloaded();
            (
                num_downloaded as u64,
                format!(
                    "{} {}",
                    num_downloaded.to_string().bold(),
                    "emails".dimmed(),
                ),
            )
        },
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: false },
    )
}
