mod emls;
mod msgs;

use anyhow::Result;
use colored::Colorize;
use reinfer_client::resources::bucket::FullName as BucketFullName;
use reinfer_client::resources::documents::Document;
use reinfer_client::{Client, NewEmail, Source, TransformTag};
use scoped_threadpool::Pool;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use structopt::StructOpt;

use crate::progress::{Options as ProgressOptions, Progress};

use self::emls::ParseEmlArgs;
use self::msgs::ParseMsgArgs;

#[derive(Debug, StructOpt)]
pub enum ParseArgs {
    #[structopt(name = "msgs")]
    /// Parse unicode msg files. Note: Currently the body is processed as plain text.
    /// Html bodies are not supported.
    Msgs(ParseMsgArgs),

    #[structopt(name = "emls")]
    /// Parse eml files.
    /// Html bodies are not supported.
    Emls(ParseEmlArgs),
}

pub fn run(args: &ParseArgs, client: Client, pool: &mut Pool) -> Result<()> {
    match args {
        ParseArgs::Msgs(parse_msg_args) => msgs::parse(&client, parse_msg_args),
        ParseArgs::Emls(parse_eml_args) => emls::parse(&client, parse_eml_args, pool),
    }
}

pub struct Statistics {
    processed: AtomicUsize,
    failed: AtomicUsize,
    uploaded: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            processed: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            uploaded: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_uploaded(&self, num_uploaded: usize) {
        self.uploaded.fetch_add(num_uploaded, Ordering::SeqCst);
    }

    #[inline]
    fn increment_failed(&self) {
        self.failed.fetch_add(1, Ordering::SeqCst);
    }

    #[inline]
    fn increment_processed(&self) {
        self.processed.fetch_add(1, Ordering::SeqCst);
    }

    #[inline]
    fn num_uploaded(&self) -> usize {
        self.uploaded.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed(&self) -> usize {
        self.failed.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_processed(&self) -> usize {
        self.processed.load(Ordering::SeqCst)
    }
}

pub fn get_files_in_directory(
    directory: &PathBuf,
    target_extension: &str,
) -> Result<Vec<DirEntry>> {
    Ok(std::fs::read_dir(directory)?
        .filter_map(|path| {
            let path = path.ok()?;
            if path.path().extension().is_some_and(|extension| {
                *extension.to_ascii_lowercase() == *target_extension.to_ascii_lowercase()
            }) {
                Some(path)
            } else {
                None
            }
        })
        .collect())
}

fn upload_batch_of_new_emails(
    client: &Client,
    bucket: &BucketFullName,
    emails: &[NewEmail],
    no_charge: bool,
    statistics: &Arc<Statistics>,
) -> Result<()> {
    client.put_emails(bucket, emails, no_charge)?;
    statistics.add_uploaded(emails.len());
    Ok(())
}

fn upload_batch_of_documents(
    client: &Client,
    source: &Source,
    documents: &[Document],
    transform_tag: &TransformTag,
    no_charge: bool,
    statistics: &Arc<Statistics>,
) -> Result<()> {
    client.sync_raw_emails(
        &source.full_name(),
        documents,
        transform_tag,
        false,
        no_charge,
    )?;
    statistics.add_uploaded(documents.len());
    Ok(())
}

fn get_progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistic| {
            let num_processed = statistic.num_processed();
            let num_failed = statistic.num_failed();
            let num_uploaded = statistic.num_uploaded();
            (
                num_processed as u64,
                format!(
                    "{} {} {} {} {} {}",
                    num_processed.to_string().bold(),
                    "processed".dimmed(),
                    num_failed.to_string().bold(),
                    "failed".dimmed(),
                    num_uploaded.to_string().bold(),
                    "uploaded".dimmed()
                ),
            )
        },
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: false },
    )
}
