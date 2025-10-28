//! Commands for creating and uploading emails to buckets
//!
//! This module provides functionality to:
//! - Upload emails from files or stdin to buckets
//! - Handle batch processing for efficiency
//! - Track upload progress and statistics
//! - Support error recovery with resume-on-error functionality

// Standard library imports
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

// External crate imports
use anyhow::{Context, Result};
use colored::Colorize;
use log::info;
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{configuration::Configuration, emails_api::add_emails_to_bucket},
    models::{AddEmailsToBucketRequest, EmailNew},
};

// Local crate imports
use crate::{
    commands::ensure_uip_user_consents_to_ai_unit_charge,
    progress::{Options as ProgressOptions, Progress},
    utils::{resolve_bucket, types::identifiers::BucketIdentifier},
};

/// Command line arguments for uploading emails to buckets
#[derive(Debug, StructOpt)]
pub struct CreateEmailsArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with emails. If not specified, stdin will be used.
    emails_path: Option<PathBuf>,

    #[structopt(short = "b", long = "bucket")]
    /// Name of the bucket where the emails will be uploaded.
    bucket: BucketIdentifier,

    #[structopt(long = "batch-size", default_value = "128")]
    /// Number of emails to batch in a single request.
    batch_size: usize,

    #[structopt(long)]
    /// Don't display a progress bar (only applicable when --file is used).
    no_progress: bool,

    #[structopt(short = "n", long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,

    #[structopt(short = "y", long = "yes")]
    /// Consent to ai unit charge. Suppresses confirmation prompt.
    yes: bool,

    #[structopt(long = "resume-on-error")]
    /// Whether to attempt to resume processing on error
    resume_on_error: bool,
}

/// Upload emails from file or stdin to a specified bucket
///
/// This function handles:
/// - User consent for AI unit charges
/// - Bucket resolution from name or ID
/// - Progress tracking for file uploads
/// - Batch processing with error recovery
pub fn create(config: &Configuration, args: &CreateEmailsArgs) -> Result<()> {
    if !args.no_charge && !args.yes {
        ensure_uip_user_consents_to_ai_unit_charge(&config.base_path.parse()?)?;
    }

    let bucket = resolve_bucket(config, &args.bucket)?;

    let statistics = match &args.emails_path {
        Some(emails_path) => {
            info!(
                "Uploading emails from file `{}` to bucket `{}/{}` [id: {}]",
                emails_path.display(),
                bucket.owner,
                bucket.name,
                bucket.id,
            );
            let file_metadata = fs::metadata(emails_path).with_context(|| {
                format!(
                    "Could not get file metadata for `{}`",
                    emails_path.display()
                )
            })?;
            let file = BufReader::new(
                File::open(emails_path)
                    .with_context(|| format!("Could not open file `{}`", emails_path.display()))?,
            );
            let statistics = Arc::new(Statistics::new());
            let progress = if args.no_progress {
                None
            } else {
                Some(progress_bar(file_metadata.len(), &statistics))
            };
            upload_emails_from_reader(
                config,
                &bucket,
                file,
                args.batch_size,
                &statistics,
                args.no_charge,
                args.resume_on_error,
            )?;
            if let Some(mut progress) = progress {
                progress.done();
            }
            Arc::try_unwrap(statistics).unwrap()
        }
        None => {
            info!(
                "Uploading emails from stdin to bucket `{}/{}` [id: {}]",
                bucket.owner, bucket.name, bucket.id,
            );
            let statistics = Statistics::new();
            upload_emails_from_reader(
                config,
                &bucket,
                BufReader::new(io::stdin()),
                args.batch_size,
                &statistics,
                args.no_charge,
                args.resume_on_error,
            )?;
            statistics
        }
    };

    info!("Successfully uploaded {} emails", statistics.num_uploaded());

    Ok(())
}

/// Upload emails from a reader (file or stdin) in batches with error handling
fn upload_emails_from_reader(
    config: &Configuration,
    bucket: &openapi::models::Bucket,
    mut emails: impl BufRead,
    batch_size: usize,
    statistics: &Statistics,
    no_charge: bool,
    resume_on_error: bool,
) -> Result<()> {
    assert!(batch_size > 0);
    let mut line_number = 1;
    let mut line = String::new();
    let mut batch = Vec::with_capacity(batch_size);
    let mut eof = false;
    while !eof {
        line.clear();
        let bytes_read = emails
            .read_line(&mut line)
            .with_context(|| format!("Could not read line {line_number} from input stream"))?;

        if bytes_read == 0 {
            eof = true;
        } else {
            statistics.add_bytes_read(bytes_read);
            let new_email =
                serde_json::from_str::<EmailNew>(line.trim_end()).with_context(|| {
                    format!("Could not parse email at line {line_number} from input stream")
                })?;
            batch.push(new_email);
        }

        if batch.len() == batch_size || (!batch.is_empty() && eof) {
            // Upload emails

            if resume_on_error {
                // Use OpenAPI split-on-failure for resilience
                let result = crate::utils::openapi::split_failure::execute_with_split_on_failure(
                    |req| {
                        add_emails_to_bucket(
                            config,
                            &bucket.owner,
                            &bucket.name,
                            req,
                            Some(no_charge),
                        )
                    },
                    AddEmailsToBucketRequest::new(batch.to_vec()),
                    "add_emails_to_bucket",
                )
                .context("Could not upload batch of emails")?;

                statistics.add_emails(StatisticsUpdate {
                    uploaded: batch.len() - result.num_failed,
                    failed: result.num_failed,
                });
                batch.clear();
            } else {
                let request = AddEmailsToBucketRequest::new(batch.to_vec());
                add_emails_to_bucket(
                    config,
                    &bucket.owner,
                    &bucket.name,
                    request,
                    Some(no_charge),
                )
                .context("Could not upload batch of emails")?;
                statistics.add_emails(StatisticsUpdate {
                    uploaded: batch.len(),
                    failed: 0,
                });
                batch.clear();
            }
        }

        line_number += 1;
    }

    Ok(())
}

// ============================================================================
// Statistics Tracking
// ============================================================================

/// Update data for email upload statistics
#[derive(Debug)]
pub struct StatisticsUpdate {
    uploaded: usize,
    failed: usize,
}

/// Thread-safe statistics tracking for email uploads
#[derive(Debug)]
pub struct Statistics {
    bytes_read: AtomicUsize,
    uploaded: AtomicUsize,
    failed: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            bytes_read: AtomicUsize::new(0),
            uploaded: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_bytes_read(&self, bytes_read: usize) {
        self.bytes_read.fetch_add(bytes_read, Ordering::SeqCst);
    }

    #[inline]
    fn add_emails(&self, update: StatisticsUpdate) {
        self.uploaded.fetch_add(update.uploaded, Ordering::SeqCst);
        self.failed.fetch_add(update.failed, Ordering::SeqCst);
    }

    #[inline]
    fn bytes_read(&self) -> usize {
        self.bytes_read.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_uploaded(&self) -> usize {
        self.uploaded.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed(&self) -> usize {
        self.failed.load(Ordering::SeqCst)
    }
}

/// Create a progress bar for email upload tracking with detailed statistics
fn progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let bytes_read = statistics.bytes_read();
            let num_uploaded = statistics.num_uploaded();
            let num_failed = statistics.num_failed();

            let failed_emails_string = if num_failed > 0 {
                format!(" {num_failed} {}", "skipped".dimmed())
            } else {
                String::new()
            };

            (
                bytes_read as u64,
                format!(
                    "{} {}{}",
                    num_uploaded.to_string().bold(),
                    "emails".dimmed(),
                    failed_emails_string
                ),
            )
        },
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: true },
    )
}
