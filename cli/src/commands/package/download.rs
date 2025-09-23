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
use itertools::Itertools;
use openapi::{
    apis::{
        configuration::Configuration,
        datasets_api::{get_dataset, get_dataset_statistics},
        sources_api::get_source,
        buckets_api::get_bucket,
        comments_api::query_comments,
        emails_api::get_bucket_emails,
    },
    models::{
        Dataset, DatasetFullName, Source, Bucket, CommentFilter,
        QueryCommentsRequest, QueryCommentsResponse, Order, QueryCommentsOrderRecent,
        GetDatasetStatisticsRequest, GetBucketEmailsRequest,
    },
};
use reinfer_client::{
    resources::{
        attachments::AttachmentMetadata,
        dataset::DatasetFlag,
    },
    CommentId, DatasetName, HasAnnotations, SourceId,
};
use scoped_threadpool::Pool;
use structopt::StructOpt;

use crate::{
    commands::package::{
        AttachmentKey, CommentBatchKey, EmailBatchKey, PackageContentId, PackageWriter,
    },
    progress::Progress,
    DEFAULT_PROJECT_NAME,
};
use colored::Colorize;

#[derive(Debug)]
struct IxpDatasetIdentifier(DatasetFullName);

impl FromStr for IxpDatasetIdentifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let identifier = match DatasetFullName::from_str(s) {
            Ok(full_name) => IxpDatasetIdentifier(full_name),
            Err(_) => {
                let full_name =
                    DatasetName(s.to_string()).with_project(&DEFAULT_PROJECT_NAME.clone())?;

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
    let bucket_full_name = bucket.full_name();
    
    let mut request = GetBucketEmailsRequest::new();
    request.limit = Some(batch_size as i32);
    
    let mut idx = 0;
    loop {
        let response = get_bucket_emails(config, &bucket_full_name.owner, &bucket_full_name.name, request.clone())
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
    let mut request = QueryCommentsRequest {
        continuation: None,
        limit: Some(batch_size as i32),
        attribute_filters: None,
        collapse_mode: None,
        filter: Some(CommentFilter {
            sources: Some(vec![source_id.clone()]),
            ..Default::default()
        }),
        order: Box::new(Order::Recent(Box::new(QueryCommentsOrderRecent {
            kind: openapi::models::_query_comments_order_recent::Kind::Recent,
        }))),
    };

    let mut documents: Vec<(AttachmentMetadata, AttachmentKey, SourceId, CommentId)> = Vec::new();
    let mut batch_idx = 0;

    // Use OpenAPI pagination pattern
    loop {
        let response = query_comments(
            config,
            &dataset.owner,
            &dataset.name,
            request.clone(),
            None, // limit is already in the request
            request.continuation.as_deref(),
            None, // collapse_mode is already in the request
            None, // order is already in the request
        )
        .context("Failed to query comments")?;

        if response.results.is_empty() {
            break;
        }

        let comments = response.results;

        package_writer
            .write_comment_batch(source_id, CommentBatchKey(batch_idx), &comments)
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
                                    source_id.clone(),
                                    comment.comment.id.clone(),
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

        batch_idx += 1;

        // Update continuation for next iteration
        request.continuation = response.continuation;
        if request.continuation.is_none() {
            break;
        }
    }
    package_batch_of_documents(&documents, statistics, package_writer, config, pool)
}

fn should_package_batch_of_documents(
    documents: &[(AttachmentMetadata, AttachmentKey, SourceId, CommentId)],
    max_attachment_memory_mb: &u64,
) -> bool {
    documents
        .iter()
        .map(|(attachment, _, _, _)| attachment.size)
        .sum::<u64>()
        / 1_000_000
        >= *max_attachment_memory_mb
}

fn package_batch_of_documents(
    documents: &[(AttachmentMetadata, AttachmentKey, SourceId, CommentId)],
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
                scope.execute(move || {
                    // TODO: Need to implement OpenAPI equivalent for IXP documents
                    let result = client.get_ixp_document(source_id, comment_id);

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
                                        source_id,
                                        comment_id,
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
    let dataset_full_name = dataset.full_name();
    let request = GetDatasetStatisticsRequest::new(Vec::new());
    let response = get_dataset_statistics(config, &dataset_full_name.owner, &dataset_full_name.name, request)
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
            let response = get_source(config, &source_id.owner, &source_id.name)?;
            Ok(response.source.bucket_id)
        })
        .try_collect::<Option<BucketId>, Vec<Option<BucketId>>, anyhow::Error>()?
        .into_iter()
        .flatten()
        .map(|bucket_id| {
            let response = get_bucket(config, &bucket_id.owner, &bucket_id.name)?;
            Ok(response.bucket)
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
        let response = get_source(config, &source_id.owner, &source_id.name)
            .context("Failed to get source")?;
        let source = response.source;

        package_writer
            .write_source(&source)
            .context("Failed to write source")?;

        package_source_contents(
            &dataset.full_name(),
            source_id,
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
        let response = get_source(config, &source_id.owner, &source_id.name)
            .context("Failed to get source")?;
        let source = response.source;

        package_writer
            .write_source(&source)
            .context("Failed to write source")?;

        package_source_contents(
            &dataset.full_name(),
            source_id,
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
    let response = get_dataset(config, &project.0.owner, &project.0.name).context(format!(
        "Could not get project with the name {0}",
        project.0
    ))?;
    let dataset = response.dataset;

    if dataset.has_flag(DatasetFlag::Ixp) {
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
