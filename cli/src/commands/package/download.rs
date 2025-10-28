use std::{
    fs::remove_file,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::channel,
        Arc,
    },
};

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use itertools::Itertools;
use scoped_threadpool::Pool;
use structopt::StructOpt;

use openapi::{
    apis::{
        configuration::Configuration,
        datasets_api::{get_dataset, get_dataset_statistics},
        emails_api::get_bucket_emails,
    },
    models::{
        Attachment, Bucket, CommentFilter, Dataset, DatasetFlag, GetBucketEmailsRequest,
        GetDatasetStatisticsRequest, Order, QueryCommentsOrderRecent, QueryCommentsRequest,
    },
};

use crate::{
    commands::package::{
        AttachmentKey, CommentBatchKey, EmailBatchKey, PackageContentId, PackageWriter,
    },
    progress::Progress,
    utils::{
        conversions::HasAnnotations, get_dataset_query_iter, get_document_bytes,
        types::names::FullName as DatasetFullName, AttachmentExt, CommentId, SourceId,
    },
    DEFAULT_PROJECT_NAME,
};

// Type aliases for compatibility
pub type BucketId = String;

#[derive(Debug)]
struct IxpDatasetIdentifier(DatasetFullName);

impl FromStr for IxpDatasetIdentifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let identifier = match DatasetFullName::from_str(s) {
            Ok(full_name) => IxpDatasetIdentifier(full_name),
            Err(_) => {
                // Create full name directly: "project/dataset"
                let full_name_str = format!("{}/{}", DEFAULT_PROJECT_NAME.as_str(), s);
                let full_name = DatasetFullName::from_str(&full_name_str)?;
                IxpDatasetIdentifier(full_name)
            }
        };
        Ok(identifier)
    }
}

#[derive(Debug, StructOpt)]
pub struct DownloadPackageArgs {
    /// The api name of the Unstructued Docs project / CM dataset
    project: IxpDatasetIdentifier,

    /// Path to save the package to
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file: PathBuf,

    #[structopt(long)]
    /// Whether to overwrite existing package file
    overwrite: bool,

    #[structopt(long = "batch-size", default_value = "128")]
    /// Number of comments to batch in a single request.
    batch_size: usize,

    #[structopt(long = "max-attachment-memory-mb", default_value = "256")]
    /// Maximum number of mb to use to store attachments in memory
    max_attachment_memory_mb: u64,
}

#[allow(clippy::too_many_arguments)]
fn package_bucket_contents(
    bucket: &Bucket,
    batch_size: usize,
    package_writer: &mut PackageWriter,
    config: &Configuration,
    statistics: &Arc<Statistics>,
) -> Result<()> {
    let mut request = GetBucketEmailsRequest::new();
    request.limit = Some(batch_size as i32);

    let mut idx = 0;
    loop {
        let response = get_bucket_emails(config, &bucket.owner, &bucket.name, request.clone())
            .context("Failed to get emails from bucket")?;

        if response.emails.is_empty() {
            break;
        }

        package_writer.write_email_batch(&bucket.id, EmailBatchKey(idx), &response.emails)?;
        statistics.add_emails(response.emails.len());
        idx += 1;

        // Check if there are more pages
        if let Some(continuation) = response.continuation {
            request.continuation = Some(continuation);
        } else {
            break;
        }
    }

    Ok(())
}

enum SourceType {
    CommunicationsMining,
    UnstructedAndComplexDocs,
}

#[allow(clippy::too_many_arguments)]
fn package_source_contents(
    dataset: &DatasetFullName,
    source_id: &SourceId,
    batch_size: usize,
    package_writer: &mut PackageWriter,
    config: &Configuration,
    statistics: &Arc<Statistics>,
    max_attachment_memory_mb: &u64,
    pool: &mut Pool,
    source_type: SourceType,
) -> Result<()> {
    let request = QueryCommentsRequest {
        continuation: None,
        limit: Some(batch_size as i32),
        attribute_filters: None,
        collapse_mode: None,
        filter: Some(CommentFilter {
            sources: Some(vec![source_id.0.clone()]),
            ..Default::default()
        }),
        order: Box::new(Order::Recent(Box::new(QueryCommentsOrderRecent {
            kind: openapi::models::_query_comments_order_recent::Kind::Recent,
        }))),
    };

    let source_comments_iter = get_dataset_query_iter(
        config.clone(),
        dataset.owner().to_string(),
        dataset.name().to_string(),
        request,
    );

    let mut documents: Vec<(Attachment, AttachmentKey, String, CommentId)> = Vec::new();

    source_comments_iter
        .enumerate()
        .try_for_each(|(idx, result)| -> Result<()> {
            let comments = result.context("Failed to read comment batch")?;

            package_writer
                .write_comment_batch(source_id.as_ref(), CommentBatchKey(idx), &comments)
                .context("Failed to write comment batch")?;

            statistics.add_comments(comments.len());
            statistics.add_annotations(comments.iter().filter(|c| c.has_annotations()).count());

            if matches!(source_type, SourceType::UnstructedAndComplexDocs) {
                documents.append(
                    &mut comments
                        .iter()
                        .flat_map(|comment| {
                            comment
                                .comment
                                .clone()
                                .attachments
                                .into_iter()
                                .enumerate()
                                .map(|(idx, attachment)| {
                                    (
                                        attachment,
                                        AttachmentKey(idx),
                                        source_id.0.clone(),
                                        CommentId(comment.comment.id.clone()),
                                    )
                                })
                        })
                        .collect(),
                );

                if should_package_batch_of_documents(&documents, max_attachment_memory_mb) {
                    package_batch_of_documents(
                        &documents,
                        statistics,
                        package_writer,
                        config,
                        pool,
                    )
                    .context("Failed to package document batch")?;
                    documents.clear();
                }
            }

            Ok(())
        })?;
    package_batch_of_documents(&documents, statistics, package_writer, config, pool)
}

fn should_package_batch_of_documents(
    documents: &[(Attachment, AttachmentKey, String, CommentId)],
    max_attachment_memory_mb: &u64,
) -> bool {
    documents
        .iter()
        .map(|(attachment, _, _, _)| attachment.size as u64)
        .sum::<u64>()
        / 1_000_000
        >= *max_attachment_memory_mb
}

fn package_batch_of_documents(
    documents: &[(Attachment, AttachmentKey, String, CommentId)],
    statistics: &Arc<Statistics>,
    package_writer: &mut PackageWriter,
    config: &Configuration,
    pool: &mut Pool,
) -> Result<()> {
    let (error_sender, error_receiver) = channel();
    let (document_sender, document_receiver) = channel();

    pool.scoped(|scope| {
        documents
            .iter()
            .for_each(|(metadata, key, source_id, comment_id)| {
                let error_sender = error_sender.clone();
                let sender = document_sender.clone();
                let config = config.clone();
                scope.execute(move || {
                    let result =
                        get_document_bytes(&config, source_id.as_ref(), comment_id.0.as_str());

                    let extension = metadata.extension();

                    match result {
                        Err(err) => {
                            error_sender.send(err).expect("Could not send error");
                        }
                        Ok(attachment) => {
                            statistics.add_document_download(1);

                            sender
                                .send((
                                    PackageContentId::Document {
                                        source_id: source_id.as_ref(),
                                        comment_id: comment_id.as_ref(),
                                        key,
                                        extension,
                                    },
                                    attachment,
                                ))
                                .expect("could not send attachment");
                        }
                    }
                });
            });
    });

    if let Ok(error) = error_receiver.try_recv() {
        return Err(anyhow!("Failed to download document {}", error));
    };

    drop(document_sender);

    document_receiver
        .iter()
        .try_for_each(|(content_id, content)| -> Result<()> {
            package_writer
                .write_bytes(content_id, &content)
                .context("Failed to write document bytes")?;
            statistics.add_document_write(1);
            Ok(())
        })?;

    Ok(())
}

fn get_comment_count(dataset: &Dataset, config: &Configuration) -> Result<u64> {
    let request = GetDatasetStatisticsRequest::new(Vec::new());
    let response = get_dataset_statistics(config, &dataset.owner, &dataset.name, request)
        .context("Failed to get dataset statistics")?;
    Ok(response.statistics.num_comments as u64)
}

fn package_cm_project(
    config: &Configuration,
    dataset: &Dataset,
    file: &PathBuf,
    batch_size: &usize,
    max_attachment_memory_mb: &u64,
    pool: &mut Pool,
) -> Result<()> {
    let total_comments =
        get_comment_count(dataset, config).context("Failed to get comment count")?;

    let mut package_writer =
        PackageWriter::new(file.into()).context("Failed to create new package writer")?;

    // Write dataset
    package_writer
        .write_dataset(dataset)
        .context("Failed to write dataset")?;

    // Get buckets
    let buckets: Vec<Bucket> = dataset
        .source_ids
        .iter()
        .map(|source_id| -> Result<Option<BucketId>> {
            let response = openapi::apis::sources_api::get_source_by_id(config, source_id)?;
            Ok(response.source.bucket_id)
        })
        .try_collect::<Option<BucketId>, Vec<Option<BucketId>>, anyhow::Error>()?
        .into_iter()
        .flatten()
        .map(|bucket_id| -> Result<Bucket> {
            let response = openapi::apis::buckets_api::get_bucket_by_id(config, &bucket_id)?;
            Ok(*response.bucket)
        })
        .try_collect()?;

    // Get Progress Bar
    let statistics = Arc::new(Statistics::new());
    let _progress_bar = get_cm_progress_bar(total_comments, !buckets.is_empty(), &statistics);

    // Write buckets and raw emails
    for bucket in buckets {
        package_writer.write_bucket(&bucket)?;
        package_bucket_contents(
            &bucket,
            *batch_size,
            &mut package_writer,
            config,
            &statistics,
        )?;
    }

    // Write sources and comments
    for source_id in &dataset.source_ids {
        let response = openapi::apis::sources_api::get_source_by_id(config, source_id)
            .context("Failed to get source")?;
        let source = response.source;

        package_writer
            .write_source(&source)
            .context("Failed to write source")?;

        package_source_contents(
            &DatasetFullName(format!("{}/{}", dataset.owner, dataset.name)),
            &SourceId(source_id.clone()),
            *batch_size,
            &mut package_writer,
            config,
            &statistics,
            max_attachment_memory_mb,
            pool,
            SourceType::CommunicationsMining,
        )?;
    }
    Ok(())
}

fn package_ixp_project(
    config: &Configuration,
    dataset: &Dataset,
    file: &PathBuf,
    batch_size: &usize,
    max_attachment_memory_mb: &u64,
    pool: &mut Pool,
) -> Result<()> {
    let total_comments =
        get_comment_count(dataset, config).context("Failed to get comment count")?;

    let statistics = Arc::new(Statistics::new());
    let _progress_bar = get_ixp_progress_bar(total_comments, &statistics);

    // Write dataset
    let mut package_writer =
        PackageWriter::new(file.into()).context("Failed to create new package writer")?;
    package_writer
        .write_dataset(dataset)
        .context("Failed to write dataset")?;

    // Package each source
    for source_id in &dataset.source_ids {
        let response = openapi::apis::sources_api::get_source_by_id(config, source_id)
            .context("Failed to get source")?;
        let source = response.source;

        package_writer
            .write_source(&source)
            .context("Failed to write source")?;

        package_source_contents(
            &DatasetFullName(format!("{}/{}", dataset.owner, dataset.name)),
            &SourceId(source_id.clone()),
            *batch_size,
            &mut package_writer,
            config,
            &statistics,
            max_attachment_memory_mb,
            pool,
            SourceType::UnstructedAndComplexDocs,
        )
        .context("Failed to package source context")?;
    }

    package_writer
        .finish()
        .context("Failed to finish package writer")?;
    log::info!("Package exported");
    Ok(())
}

pub fn run(args: &DownloadPackageArgs, config: &Configuration, pool: &mut Pool) -> Result<()> {
    let DownloadPackageArgs {
        project,
        file,
        overwrite,
        batch_size,
        max_attachment_memory_mb,
    } = args;
    // Delete output file if overwriting
    if *overwrite && file.is_file() {
        remove_file(file).context("Failed to remove existing file")?;
    }

    // Get dataset, statistics and progres bar
    let response = get_dataset(config, project.0.owner(), project.0.name()).context(format!(
        "Could not get project with the name {0}",
        project.0
    ))?;
    let dataset = response.dataset;

    if dataset._dataset_flags.contains(&DatasetFlag::Ixp) {
        package_ixp_project(
            config,
            &dataset,
            file,
            batch_size,
            max_attachment_memory_mb,
            pool,
        )
    } else {
        package_cm_project(
            config,
            &dataset,
            file,
            batch_size,
            max_attachment_memory_mb,
            pool,
        )
    }
}

#[derive(Debug)]
pub struct Statistics {
    comments: AtomicUsize,
    emails: AtomicUsize,
    document_downloads: AtomicUsize,
    document_writes: AtomicUsize,
    annotations: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            comments: AtomicUsize::new(0),
            emails: AtomicUsize::new(0),
            annotations: AtomicUsize::new(0),
            document_downloads: AtomicUsize::new(0),
            document_writes: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_comments(&self, num: usize) {
        self.comments.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_emails(&self, num: usize) {
        self.emails.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_document_download(&self, num: usize) {
        self.document_downloads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_document_write(&self, num: usize) {
        self.document_writes.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_annotations(&self, num: usize) {
        self.annotations.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn num_comments(&self) -> usize {
        self.comments.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_emails(&self) -> usize {
        self.emails.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_document_downloaded(&self) -> usize {
        self.document_downloads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_document_writes(&self) -> usize {
        self.document_writes.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_annotations(&self) -> usize {
        self.annotations.load(Ordering::SeqCst)
    }
}

fn get_cm_progress_bar(
    total_comments: u64,
    has_buckets: bool,
    statistics: &Arc<Statistics>,
) -> Progress {
    let total = if has_buckets {
        total_comments * 2
    } else {
        total_comments
    };

    Progress::new(
        move |statistics| {
            let num_comments = statistics.num_comments();
            let num_annotations = statistics.num_annotations();
            let num_emails = statistics.num_emails();
            (
                (num_comments + num_emails) as u64,
                format!(
                    "{} {} {} {} {} {}",
                    num_comments.to_string().bold(),
                    "comments".dimmed(),
                    num_annotations.to_string().bold(),
                    "annotations".dimmed(),
                    num_emails.to_string().bold(),
                    "emails downloaded".dimmed(),
                ),
            )
        },
        statistics,
        Some(total),
        crate::progress::Options { bytes_units: false },
    )
}

fn get_ixp_progress_bar(total_comments: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let num_comments = statistics.num_comments();
            let num_docs_downloaded = statistics.num_document_downloaded();
            let num_annotations = statistics.num_annotations();
            let num_docs_written = statistics.num_document_writes();
            (
                ((num_comments + num_docs_downloaded + num_docs_written) / 3) as u64,
                format!(
                    "{} {} {} {} {} {} {} {}",
                    num_comments.to_string().bold(),
                    "comments".dimmed(),
                    num_annotations.to_string().bold(),
                    "annotations".dimmed(),
                    num_docs_downloaded.to_string().bold(),
                    "docs downloaded".dimmed(),
                    num_docs_written.to_string().bold(),
                    "docs written".dimmed(),
                ),
            )
        },
        statistics,
        Some(total_comments),
        crate::progress::Options { bytes_units: false },
    )
}
