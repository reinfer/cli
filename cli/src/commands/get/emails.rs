use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::{Context, Result};
use colored::Colorize;
use structopt::StructOpt;

use openapi::{
    apis::{
        buckets_api::get_bucket_statistics,
        configuration::Configuration,
        emails_api::{get_bucket_emails, get_email_from_bucket_by_id},
    },
    models::{Count, GetBucketEmailsRequest},
};

use crate::{
    printer::print_resources_as_json,
    progress::{Options as ProgressOptions, Progress},
    utils::{resolve_bucket, BucketIdentifier},
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

/// Download emails from a bucket, either a specific email by ID or all emails
pub fn get_many(config: &Configuration, args: &GetManyEmailsArgs) -> Result<()> {
    let GetManyEmailsArgs { bucket, path, id } = args;

    if let Some(email_id) = id {
        download_single_email(config, bucket, email_id, path)
    } else {
        download_all_emails(config, bucket, path)
    }
}

/// Download a single email by ID
fn download_single_email(
    config: &Configuration,
    bucket_identifier: &BucketIdentifier,
    email_id: &str,
    output_path: &Option<PathBuf>,
) -> Result<()> {
    match output_path {
        Some(path) => {
            let file = create_output_file(path)?;
            download_email(config, bucket_identifier, email_id, file)
        }
        None => download_email(config, bucket_identifier, email_id, io::stdout().lock()),
    }
}

/// Download all emails from a bucket
fn download_all_emails(
    config: &Configuration,
    bucket_identifier: &BucketIdentifier,
    output_path: &Option<PathBuf>,
) -> Result<()> {
    match output_path {
        Some(path) => {
            let file = create_output_file(path)?;
            download_emails(config, bucket_identifier, file)
        }
        None => download_emails(config, bucket_identifier, io::stdout().lock()),
    }
}

/// Create a buffered file writer for output
fn create_output_file(path: &PathBuf) -> Result<BufWriter<File>> {
    File::create(path)
        .with_context(|| format!("Could not open file for writing `{}`", path.display()))
        .map(BufWriter::new)
}

/// Download a specific email by ID
fn download_email(
    config: &Configuration,
    bucket_identifier: &BucketIdentifier,
    email_id: &str,
    mut writer: impl Write,
) -> Result<()> {
    let bucket = resolve_bucket(config, bucket_identifier)?;

    let response = get_email_from_bucket_by_id(config, &bucket.owner, &bucket.name, email_id)
        .with_context(|| {
            format!(
                "Failed to get email {} from bucket {}/{}",
                email_id, bucket.owner, bucket.name
            )
        })?;

    print_resources_as_json(response.emails.into_iter(), &mut writer)
}

/// Download all emails from a bucket with pagination
fn download_emails(
    config: &Configuration,
    bucket_identifier: &BucketIdentifier,
    mut writer: impl Write,
) -> Result<()> {
    let bucket = resolve_bucket(config, bucket_identifier)?;
    let statistics = Arc::new(Statistics::new());

    let progress_bytes = get_bucket_email_count(config, &bucket)?;
    let _progress = create_progress_bar(progress_bytes, &statistics);

    fetch_all_emails_with_pagination(config, &bucket, &mut writer, &statistics)?;

    log::info!(
        "Successfully downloaded {} emails.",
        statistics.num_downloaded(),
    );
    Ok(())
}

/// Get the total count of emails in the bucket for progress tracking
fn get_bucket_email_count(config: &Configuration, bucket: &openapi::models::Bucket) -> Result<u64> {
    let bucket_statistics = get_bucket_statistics(config, &bucket.owner, &bucket.name)
        .with_context(|| {
            format!(
                "Failed to get statistics for bucket {}/{}",
                bucket.owner, bucket.name
            )
        })?;

    let count = match *bucket_statistics.statistics.count {
        Count::LowerBound(lower_bound) => lower_bound.value,
        Count::Exact(exact) => exact.value,
    } as u64;

    Ok(count)
}

/// Fetch all emails using pagination
fn fetch_all_emails_with_pagination(
    config: &Configuration,
    bucket: &openapi::models::Bucket,
    writer: &mut impl Write,
    statistics: &Arc<Statistics>,
) -> Result<()> {
    let mut continuation: Option<String> = None;
    const EMAILS_PER_PAGE: i32 = 100;

    loop {
        let request = GetBucketEmailsRequest {
            continuation: continuation.clone(),
            limit: Some(EMAILS_PER_PAGE),
        };

        let response = get_bucket_emails(config, &bucket.owner, &bucket.name, request)
            .with_context(|| {
                format!(
                    "Failed to get emails from bucket {}/{}",
                    bucket.owner, bucket.name
                )
            })?;

        let emails_count = response.emails.len();
        let has_emails = !response.emails.is_empty();

        statistics.add_emails(emails_count);
        print_resources_as_json(response.emails.into_iter(), &mut *writer)?;

        // Server only returns a continuation token when len == limit
        continuation = response.continuation;
        if continuation.is_none() || !has_emails {
            break;
        }
    }

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

/// Create a progress bar for email download tracking
fn create_progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
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
