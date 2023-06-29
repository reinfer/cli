use anyhow::{Context, Result};
use colored::Colorize;
use log::info;
use reinfer_client::{Bucket, BucketIdentifier, Client, NewEmail};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

use crate::{
    commands::ensure_uip_user_consents_to_ai_unit_charge,
    progress::{Options as ProgressOptions, Progress},
};

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
}

pub fn create(client: &Client, args: &CreateEmailsArgs) -> Result<()> {
    if !args.no_charge && !args.yes {
        ensure_uip_user_consents_to_ai_unit_charge(client.base_url())?;
    }

    let bucket = client
        .get_bucket(args.bucket.clone())
        .with_context(|| format!("Unable to get bucket {}", args.bucket))?;

    let statistics = match &args.emails_path {
        Some(emails_path) => {
            info!(
                "Uploading emails from file `{}` to bucket `{}` [id: {}]",
                emails_path.display(),
                bucket.full_name(),
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
                client,
                &bucket,
                file,
                args.batch_size,
                &statistics,
                args.no_charge,
            )?;
            if let Some(mut progress) = progress {
                progress.done();
            }
            Arc::try_unwrap(statistics).unwrap()
        }
        None => {
            info!(
                "Uploading emails from stdin to bucket `{}` [id: {}]",
                bucket.full_name(),
                bucket.id,
            );
            let statistics = Statistics::new();
            upload_emails_from_reader(
                client,
                &bucket,
                BufReader::new(io::stdin()),
                args.batch_size,
                &statistics,
                args.no_charge,
            )?;
            statistics
        }
    };

    info!(
        concat!("Successfully uploaded {} emails",),
        statistics.num_uploaded(),
    );

    Ok(())
}

fn upload_emails_from_reader(
    client: &Client,
    bucket: &Bucket,
    mut emails: impl BufRead,
    batch_size: usize,
    statistics: &Statistics,
    no_charge: bool,
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
                serde_json::from_str::<NewEmail>(line.trim_end()).with_context(|| {
                    format!("Could not parse email at line {line_number} from input stream")
                })?;
            batch.push(new_email);
        }

        if batch.len() == batch_size || (!batch.is_empty() && eof) {
            // Upload emails
            client
                .put_emails(&bucket.full_name(), &batch, no_charge)
                .context("Could not upload batch of emails")?;
            statistics.add_emails(StatisticsUpdate {
                uploaded: batch.len(),
            });
            batch.clear();
        }

        line_number += 1;
    }

    Ok(())
}

#[derive(Debug)]
pub struct StatisticsUpdate {
    uploaded: usize,
}

#[derive(Debug)]
pub struct Statistics {
    bytes_read: AtomicUsize,
    uploaded: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            bytes_read: AtomicUsize::new(0),
            uploaded: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_bytes_read(&self, bytes_read: usize) {
        self.bytes_read.fetch_add(bytes_read, Ordering::SeqCst);
    }

    #[inline]
    fn add_emails(&self, update: StatisticsUpdate) {
        self.uploaded.fetch_add(update.uploaded, Ordering::SeqCst);
    }

    #[inline]
    fn bytes_read(&self) -> usize {
        self.bytes_read.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_uploaded(&self) -> usize {
        self.uploaded.load(Ordering::SeqCst)
    }
}

fn progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let bytes_read = statistics.bytes_read();
            let num_uploaded = statistics.num_uploaded();
            (
                bytes_read as u64,
                format!("{} {}", num_uploaded.to_string().bold(), "emails".dimmed()),
            )
        },
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: true },
    )
}
