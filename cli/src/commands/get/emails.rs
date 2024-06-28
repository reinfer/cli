use anyhow::{Context, Result};

use colored::Colorize;
use reinfer_client::{resources::bucket_statistics::Count, BucketIdentifier, Client};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
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
    /// Bucket name or id
    bucket: BucketIdentifier,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write comments as JSON. If not specified, stdout will be used.
    path: Option<PathBuf>,
}

pub fn get_many(client: &Client, args: &GetManyEmailsArgs) -> Result<()> {
    let GetManyEmailsArgs { bucket, path } = args;

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
        download_emails(client, bucket.clone(), io::stdout().lock())
    }
}

fn download_emails(
    client: &Client,
    bucket_identifier: BucketIdentifier,
    mut writer: impl Write,
) -> Result<()> {
    let bucket = client
        .get_bucket(bucket_identifier)
        .context("Operation to get bucket has failed.")?;

    let bucket_statistics = client
        .get_bucket_statistics(&bucket.full_name())
        .context("Could not get bucket statistics")?;

    let statistics = Arc::new(Statistics::new());

    let progress_bytes = match bucket_statistics.count {
        Count::LowerBoundBucketCount { value } => value,
        Count::ExactBucketCount { value } => value,
    } as u64;

    let _progress = get_emails_progress_bar(progress_bytes, &statistics);

    client
        .get_emails_iter(&bucket.full_name(), None)
        .try_for_each(|page| {
            let page = page.context("Operation to get emails has failed.")?;
            statistics.add_emails(page.len());
            print_resources_as_json(page.into_iter(), &mut writer)
        })?;
    log::info!(
        "Successfully downloaded {} emails.",
        statistics.num_downloaded(),
    );
    Ok(())
}

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

fn get_emails_progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
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
