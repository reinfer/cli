use anyhow::{Context, Result};

use colored::Colorize;
use openapi::{
    apis::{
        configuration::Configuration,
        buckets_api::{get_bucket, get_bucket_by_id, get_bucket_statistics},
        emails_api::{get_bucket_emails, get_email_from_bucket_by_id},
    },
    models::{GetBucketEmailsRequest, BucketStatistics, Count},
};
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
    utils::resource_identifier::BucketIdentifier,
};

#[derive(Debug, StructOpt)]
pub struct GetManyEmailsArgs {
    #[structopt(name = "bucket")]
    /// Bucket name or id
    bucket: BucketIdentifier,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write comments as JSON. If not specified, stdout will be used.
    path: Option<PathBuf>,

    #[structopt(name = "id")]
    /// Id of specific email to return
    id: Option<String>,
}

pub fn get_many(config: &Configuration, args: &GetManyEmailsArgs) -> Result<()> {
    let GetManyEmailsArgs { bucket, path, id } = args;

    let file = match path {
        Some(path) => Some(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?,
        ),
        None => None,
    };

    if let Some(id) = id {
        if let Some(file) = file {
            return download_email(config, bucket.clone(), id.clone(), file);
        } else {
            return download_email(config, bucket.clone(), id.clone(), io::stdout().lock());
        }
    }

    if let Some(file) = file {
        download_emails(config, bucket.clone(), file)
    } else {
        download_emails(config, bucket.clone(), io::stdout().lock())
    }
}

fn download_email(
    config: &Configuration,
    bucket_identifier: BucketIdentifier,
    id: String,
    mut writer: impl Write,
) -> Result<()> {
    let bucket = match &bucket_identifier {
        BucketIdentifier::Id(bucket_id) => {
            get_bucket_by_id(config, bucket_id)
                .context("Operation to get bucket has failed.")?
                .bucket
        }
        BucketIdentifier::FullName(full_name) => {
            get_bucket(config, full_name.owner(), full_name.name())
                .context("Operation to get bucket has failed.")?
                .bucket
        }
    };
    // TODO: Convert to use the new API once it's implemented
    let response = get_email_from_bucket_by_id(config, &bucket.owner, &bucket.name, &id)
        .context("Failed to get email")?;

    print_resources_as_json(response.email, &mut writer)
}

fn download_emails(
    config: &Configuration,
    bucket_identifier: BucketIdentifier,
    mut writer: impl Write,
) -> Result<()> {
    let bucket = match &bucket_identifier {
        BucketIdentifier::Id(bucket_id) => {
            get_bucket_by_id(config, bucket_id)
                .context("Operation to get bucket has failed.")?
                .bucket
        }
        BucketIdentifier::FullName(full_name) => {
            get_bucket(config, full_name.owner(), full_name.name())
                .context("Operation to get bucket has failed.")?
                .bucket
        }
    };

    let bucket_statistics = get_bucket_statistics(config, &bucket.owner, &bucket.name)
        .context("Could not get bucket statistics")?;

    let statistics = Arc::new(Statistics::new());

    let progress_bytes = match *bucket_statistics.statistics.count {
        Count::LowerBound(lower_bound) => lower_bound.value,
        Count::Exact(exact) => exact.value,
    } as u64;

    let _progress = get_emails_progress_bar(progress_bytes, &statistics);

    let mut continuation: Option<String> = None;

    loop {
        let request = GetBucketEmailsRequest {
            continuation: continuation.clone(),
            limit: Some(100), // Within server's allowed range
        };

        let response = get_bucket_emails(config, &bucket.owner, &bucket.name, request)
            .context("Operation to get emails has failed.")?;

        statistics.add_emails(response.emails.len());
        print_resources_as_json(response.emails.into_iter(), &mut writer)?;

        // Server only returns a continuation token when len == limit
        continuation = response.continuation;
        if continuation.is_none() || response.emails.is_empty() {
            break;
        }
    }

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
