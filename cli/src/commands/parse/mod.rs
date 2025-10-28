mod aic_classification_csv;
mod emls;
mod msgs;
mod pff;
mod pff_sys;
mod pst;

use crate::utils::{types::identifiers::BucketIdentifier, types::transform::TransformTag};
use aic_classification_csv::ParseAicClassificationCsvArgs;
use anyhow::Context;
use anyhow::Result;
use colored::Colorize;
use openapi::{
    apis::{
        buckets_api::{get_bucket, get_bucket_by_id},
        comments_api::{sync_comments, sync_raw_emails},
        configuration::Configuration,
        emails_api::add_emails_to_bucket,
    },
    models::{
        AddEmailsToBucketRequest, CommentNew, EmailNew, RawEmailDocument, Source,
        SyncCommentsRequest, SyncRawEmailsRequest,
    },
};
use pst::ParsePstArgs;
use scoped_threadpool::Pool;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use structopt::StructOpt;

use crate::progress::{Options as ProgressOptions, Progress};

use self::emls::ParseEmlArgs;
use self::msgs::ParseMsgArgs;

use super::create::annotations::AnnotationStatistic;

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

    #[structopt(name = "aic-classification-csv")]
    /// Parse a classification CSV downloaded from AI Center
    AicClassificationCsv(ParseAicClassificationCsvArgs),

    #[structopt(name = "pst")]
    /// Parse a pst
    Pst(ParsePstArgs),
}

pub fn run(args: &ParseArgs, config: &Configuration, pool: &mut Pool) -> Result<()> {
    match args {
        ParseArgs::Msgs(args) => msgs::parse(config, args),
        ParseArgs::Emls(args) => emls::parse(config, args, pool),
        ParseArgs::AicClassificationCsv(args) => aic_classification_csv::parse(config, args, pool),
        ParseArgs::Pst(args) => pst::parse(config, args),
    }
}

#[derive(Default, Debug)]
pub struct Statistics {
    processed: AtomicUsize,
    failed: AtomicUsize,
    uploaded: AtomicUsize,
    annotations: AtomicUsize,
    failed_annotations: AtomicUsize,
}

impl AnnotationStatistic for Statistics {
    fn add_annotation(&self) {
        self.annotations.fetch_add(1, Ordering::SeqCst);
    }

    fn add_failed_annotation(&self) {
        self.failed_annotations.fetch_add(1, Ordering::SeqCst);
    }
}

impl Statistics {
    fn new() -> Self {
        Self {
            ..Default::default()
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

    #[inline]
    fn num_annotations(&self) -> usize {
        self.annotations.load(Ordering::SeqCst)
    }
}

pub fn get_files_in_directory(
    directory: &PathBuf,
    target_extension: &str,
    case_insensitive: bool,
) -> Result<Vec<DirEntry>> {
    Ok(std::fs::read_dir(directory)?
        .filter_map(|path| {
            let path = path.ok()?;
            if path.path().extension().is_some_and(|extension| {
                if case_insensitive {
                    *extension.to_ascii_lowercase() == *target_extension.to_ascii_lowercase()
                } else {
                    extension == target_extension
                }
            }) {
                Some(path)
            } else {
                None
            }
        })
        .collect())
}

fn upload_batch_of_new_emails(
    config: &Configuration,
    bucket: &BucketIdentifier,
    emails: &[EmailNew],
    no_charge: bool,
    statistics: &Arc<Statistics>,
) -> Result<()> {
    // Get bucket info using OpenAPI
    let bucket_info = match bucket {
        BucketIdentifier::Id(bucket_id) => {
            let response =
                get_bucket_by_id(config, bucket_id).context("Failed to get bucket by ID")?;
            response.bucket
        }
        BucketIdentifier::FullName(full_name) => {
            let response = get_bucket(config, full_name.owner(), full_name.name())
                .context("Failed to get bucket by name")?;
            response.bucket
        }
    };

    let request = AddEmailsToBucketRequest::new(emails.to_vec());
    add_emails_to_bucket(
        config,
        &bucket_info.owner,
        &bucket_info.name,
        request,
        Some(no_charge),
    )
    .context("Failed to add emails to bucket")?;

    statistics.add_uploaded(emails.len());
    Ok(())
}

fn upload_batch_of_documents(
    config: &Configuration,
    source: &Source,
    documents: &[RawEmailDocument],
    transform_tag: &TransformTag,
    no_charge: bool,
    statistics: &Arc<Statistics>,
) -> Result<()> {
    let mut request = SyncRawEmailsRequest::new(documents.to_vec());
    request.transform_tag = Some(transform_tag.as_str().to_string());

    sync_raw_emails(
        config,
        &source.owner,
        &source.name,
        request,
        Some(no_charge),
    )
    .context("Failed to sync raw emails")?;

    statistics.add_uploaded(documents.len());
    Ok(())
}

fn upload_batch_of_comments(
    config: &Configuration,
    source: &Source,
    comments: &[CommentNew],
    no_charge: bool,
    statistics: &Statistics,
) -> Result<()> {
    let request = SyncCommentsRequest::new(comments.to_vec());

    sync_comments(
        config,
        &source.owner,
        &source.name,
        request,
        Some(no_charge),
    )
    .context("Failed to sync comments")?;

    statistics.add_uploaded(comments.len());
    Ok(())
}
fn get_progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistic| {
            let num_processed = statistic.num_processed();
            let num_failed = statistic.num_failed();
            let num_uploaded = statistic.num_uploaded();
            let num_annotated = statistic.num_annotations();

            let annotations_string = if num_annotated > 0 {
                format!(" {} {}", num_annotated, "annotated".dimmed())
            } else {
                String::new()
            };

            (
                num_processed as u64,
                format!(
                    "{} {} {} {} {} {}{}",
                    num_processed.to_string().bold(),
                    "processed".dimmed(),
                    num_failed.to_string().bold(),
                    "failed".dimmed(),
                    num_uploaded.to_string().bold(),
                    "uploaded".dimmed(),
                    annotations_string
                ),
            )
        },
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: false },
    )
}
