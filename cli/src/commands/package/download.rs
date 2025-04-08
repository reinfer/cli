use std::{
    fs::remove_file,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::channel,
        Arc,
    },
};

use anyhow::{anyhow, Context, Result};
use reinfer_client::{
    resources::{
        attachments::AttachmentMetadata,
        dataset::{DatasetFlag, QueryRequestParams},
    },
    Client, CommentFilter, CommentId, Dataset, DatasetFullName, DatasetIdentifier, DatasetName,
    HasAnnotations, SourceId,
};
use scoped_threadpool::Pool;
use structopt::StructOpt;

use crate::{
    commands::package::{AttachmentKey, CommentBatchKey, PackageContentId, PackageWriter},
    progress::Progress,
    DEFAULT_PROJECT_NAME,
};
use colored::Colorize;

#[derive(Debug, StructOpt)]
pub struct DownloadPackageArgs {
    /// The api name of the project
    project: DatasetName,

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

fn get_project(project: &DatasetName, client: &Client) -> Result<Dataset> {
    let default_project_name = DEFAULT_PROJECT_NAME.clone();

    let dataset = client
        .get_dataset(DatasetIdentifier::FullName(
            project.clone().with_project(&default_project_name)?,
        ))
        .context(format!(
            "Could not get project with the name {0}",
            project.0
        ))?;

    if !dataset.has_flag(DatasetFlag::Ixp) {
        return Err(anyhow!(
            "Can only get packages for Unstructured and Complex Document projects"
        ));
    }

    Ok(dataset)
}

#[allow(clippy::too_many_arguments)]
fn package_source_contents(
    dataset: &DatasetFullName,
    source_id: &SourceId,
    batch_size: usize,
    package_writer: &mut PackageWriter,
    client: &Client,
    statistics: &Arc<Statistics>,
    max_attachment_memory_mb: &u64,
    pool: &mut Pool,
) -> Result<()> {
    let mut query = QueryRequestParams {
        limit: Some(batch_size),
        filter: CommentFilter {
            sources: vec![source_id.clone()],
            ..Default::default()
        },
        ..Default::default()
    };

    let source_comments_iter = client.get_dataset_query_iter(dataset, &mut query);

    let mut documents: Vec<(AttachmentMetadata, AttachmentKey, SourceId, CommentId)> = Vec::new();

    source_comments_iter
        .enumerate()
        .try_for_each(|(idx, result)| -> Result<()> {
            let comments = result.context("Failed to read coment batch")?;

            package_writer
                .write_comment_batch(source_id, CommentBatchKey(idx), &comments)
                .context("Failed to write comment batch")?;

            statistics.add_comments(comments.len());
            statistics.add_annotations(comments.iter().filter(|c| c.has_annotations()).count());

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
                package_batch_of_documents(&documents, statistics, package_writer, client, pool)
                    .context("Failed to package document batch")?;
                documents.clear();
            }

            Ok(())
        })?;
    package_batch_of_documents(&documents, statistics, package_writer, client, pool)
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
    client: &Client,
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

fn get_comment_count(dataset: &Dataset, client: &Client) -> Result<u64> {
    let total_comments = *client
        .get_dataset_statistics(&dataset.full_name(), &Default::default())
        .context("Failed to get dataset statistics")?
        .num_comments as u64;
    Ok(total_comments)
}

pub fn run(args: &DownloadPackageArgs, client: &Client, pool: &mut Pool) -> Result<()> {
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
    let dataset = get_project(project, client).context("Failed to get project")?;
    let total_comments =
        get_comment_count(&dataset, client).context("Failed to get comment count")?;

    let statistics = Arc::new(Statistics::new());
    let _progress_bar = get_progress_bar(total_comments, &statistics);

    // Write dataset
    let mut package_writer =
        PackageWriter::new(file.into()).context("Failed to create new package writer")?;
    package_writer
        .write_dataset(&dataset)
        .context("Failed to write dataset")?;

    // Package each source
    for source_id in &dataset.source_ids {
        let source = client
            .get_source(source_id.clone())
            .context("Failed to get source")?;

        package_writer
            .write_source(&source)
            .context("Failed to write source")?;

        package_source_contents(
            &dataset.full_name(),
            source_id,
            *batch_size,
            &mut package_writer,
            client,
            &statistics,
            max_attachment_memory_mb,
            pool,
        )
        .context("Failed to package source context")?;
    }

    package_writer
        .finish()
        .context("Failed to finish package writer")?;
    log::info!("Package exported");
    Ok(())
}

#[derive(Debug)]
pub struct Statistics {
    comments: AtomicUsize,
    document_downloads: AtomicUsize,
    document_writes: AtomicUsize,
    annotations: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            comments: AtomicUsize::new(0),
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
fn get_progress_bar(total_comments: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let num_comments = statistics.num_comments();
            let num_docs_downloaded = statistics.num_document_downloaded();
            let num_annotations = statistics.num_annotations();
            let num_docs_written = statistics.num_document_writes();
            (
                (num_comments + num_docs_downloaded + num_docs_written) as u64,
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
