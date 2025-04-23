use colored::Colorize;
use itertools::Itertools;
use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::channel,
        Arc,
    },
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context, Result};
use reinfer_client::{
    resources::dataset::IxpDatasetNew, Client, CommentUid, Dataset, DatasetFullName, DatasetName,
    LabelDef, NewAnnotatedComment, NewLabelDef, Source, SourceId, SourceKind,
    DEFAULT_LABEL_GROUP_NAME,
};
use scoped_threadpool::Pool;
use structopt::StructOpt;

use crate::{
    commands::package::{AttachmentKey, CommentBatchKey, Package},
    progress::Progress,
};

#[derive(Debug, StructOpt)]
pub struct UploadPackageArgs {
    /// Path of the package to upload
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    #[structopt(long = "resume-on-error")]
    /// Whether to attempt to resume processing on error
    resume_on_error: bool,

    #[structopt(long = "max-attachment-memory-mb", default_value = "256")]
    /// Maximum number of mb to use to store attachments in memory
    max_attachment_memory_mb: u64,

    #[structopt(long = "dataset-creation-timeout", default_value = "30")]
    /// Maximum number of seconds to wait for dataset to be created
    dataset_creation_timeout: u64,

    #[structopt(long = "name", short = "n")]
    /// The name of the new project
    new_project_name: Option<DatasetName>,
}

fn wait_for_dataset_to_exist(dataset: &Dataset, client: &Client, timeout_s: u64) -> Result<()> {
    let start_time = Instant::now();

    while (start_time - Instant::now()).as_secs() <= timeout_s {
        let datasets = client.get_datasets()?;

        let dataset_exists = datasets
            .iter()
            .map(|dataset| dataset.full_name())
            .contains(&dataset.full_name());

        if dataset_exists {
            return Ok(());
        } else {
            sleep(Duration::from_millis(500));
        }
    }

    Err(anyhow!("Timeout waiting for dataset to be created"))
}

fn create_dataset(
    name: DatasetName,
    label_defs: Vec<LabelDef>,
    client: &Client,
    timeout_s: u64,
) -> Result<Dataset> {
    let mut new_label_defs = Vec::new();

    label_defs.iter().try_for_each(|label| -> Result<()> {
        new_label_defs.push(NewLabelDef::try_from(label)?);
        Ok(())
    })?;

    let dataset = client
        .create_ixp_dataset(IxpDatasetNew { name })
        .context("Error when creating dataset")
        .map_err(anyhow::Error::msg)?;

    wait_for_dataset_to_exist(&dataset, client, timeout_s)?;

    client
        .create_label_defs_bulk(
            &dataset.full_name(),
            DEFAULT_LABEL_GROUP_NAME.clone(),
            new_label_defs,
        )
        .context("Error when creating label defs")?;
    Ok(dataset)
}

pub struct IxpDatasetSources {
    runtime: Source,
    design_time: Source,
}

enum SourceProvider<'a> {
    Packaged(&'a mut Package),
    Remote(&'a Client),
}

impl SourceProvider<'_> {
    fn get_source(&mut self, source_id: &SourceId) -> Result<Source> {
        match self {
            Self::Packaged(package) => package.get_source_by_id(source_id),
            Self::Remote(client) => client
                .get_source(source_id.clone())
                .map_err(anyhow::Error::msg),
        }
    }
}

impl Display for SourceProvider<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Packaged(_) => "packaged",
                Self::Remote(_) => "remote",
            }
        )
    }
}

fn get_ixp_source(dataset: &Dataset, mut provider: SourceProvider) -> Result<IxpDatasetSources> {
    let mut source_by_kind: HashMap<SourceKind, Source> = HashMap::new();
    dataset
        .source_ids
        .iter()
        .try_for_each(|source_id| -> Result<()> {
            let source = provider
                .get_source(source_id)
                .context("Could not get source")?;
            if source_by_kind.contains_key(&source.kind) {
                return Err(anyhow!("Duplicate source kind found in packaged dataset"));
            }

            source_by_kind.insert(source.kind.clone(), source.clone());
            Ok(())
        })?;

    let runtime = source_by_kind
        .get(&SourceKind::IxpRuntime)
        .context(anyhow!(
            "Could not find runtime source for {0} dataset {1}",
            provider,
            dataset.id.0
        ))?
        .clone();

    let design_time = source_by_kind
        .get(&SourceKind::IxpDesignTime)
        .context(anyhow!(
            "Could not find design time source for {0} dataset {1}",
            provider,
            dataset.id.0
        ))?
        .clone();

    Ok(IxpDatasetSources {
        runtime,
        design_time,
    })
}

#[allow(clippy::too_many_arguments)]
fn unpack_ixp_source(
    old_source: &Source,
    new_source: &Source,
    new_dataset: &Dataset,
    package: &mut Package,
    client: &Client,
    pool: &mut Pool,
    resume_on_error: bool,
    statistics: &Statistics,
    max_attachment_memory_mb: &u64,
) -> Result<()> {
    let batch_count = package.get_comment_batch_count_for_source(&old_source.id);

    let mut documents = Vec::new();
    for idx in 0..batch_count {
        let batch = package
            .get_comment_batch(&old_source.id, CommentBatchKey(idx))
            .context("Could not get comment batch")?;

        batch.iter().try_for_each(|comment| -> Result<()> {
            comment.comment.attachments.iter().enumerate().try_for_each(
                |(idx, attachment)| -> Result<()> {
                    let read_result = package.read_document(
                        &old_source.id,
                        &comment.comment.id,
                        &AttachmentKey(idx),
                        attachment.extension(),
                    );

                    match read_result {
                        Ok(result) => {
                            statistics.add_document_reads(1);

                            documents.push(AnnotationWithContent {
                                comment: comment.clone(),
                                content: result,
                                size: attachment.size as usize,
                                file_name: attachment.name.clone(),
                            });

                            if should_upload_batch(&documents, max_attachment_memory_mb) {
                                upload_batch(
                                    &documents,
                                    client,
                                    &new_source.id,
                                    &new_dataset.full_name(),
                                    resume_on_error,
                                    statistics,
                                    pool,
                                )
                                .context("Failed to upload batch")?;
                                documents.clear();
                            }
                            Ok(())
                        }
                        Err(_) if resume_on_error => {
                            statistics.add_failed_document_reads(1);
                            Ok(())
                        }
                        Err(err) => Err(err),
                    }
                },
            )
        })?;
    }
    upload_batch(
        &documents,
        client,
        &new_source.id,
        &new_dataset.full_name(),
        resume_on_error,
        statistics,
        pool,
    )
}

fn should_upload_batch(batch: &[AnnotationWithContent], max_attachment_memory_mb: &u64) -> bool {
    batch
        .iter()
        .map(|attachment| attachment.size as u64)
        .sum::<u64>()
        / 1_000_000
        >= *max_attachment_memory_mb
}

struct AnnotationWithContent {
    comment: NewAnnotatedComment,
    file_name: String,
    content: Vec<u8>,
    size: usize,
}

fn upload_batch(
    batch: &Vec<AnnotationWithContent>,
    client: &Client,
    source_id: &SourceId,
    dataset_name: &DatasetFullName,
    resume_on_error: bool,
    statistics: &Statistics,
    pool: &mut Pool,
) -> Result<()> {
    let (error_sender, error_receiver) = channel();

    pool.scoped(|scope| {
        for item in batch {
            let error_sender = error_sender.clone();
            scope.execute(move || {
                let upload_result = client
                    .upload_ixp_document(source_id, item.file_name.clone(), item.content.clone())
                    .map_err(anyhow::Error::msg);

                match upload_result {
                    Ok(new_comment_id) => {
                        statistics.add_document_uploads(1);

                        let comment_uid =
                            CommentUid(format!("{}.{}", source_id.0, new_comment_id.0));

                        if item.comment.has_annotations() {
                            let labelling_result = client.update_labelling(
                                dataset_name,
                                &comment_uid,
                                None,
                                item.comment.entities.as_ref(),
                                item.comment.moon_forms.as_deref(),
                            );

                            match labelling_result {
                                Ok(_) => {
                                    statistics.add_annotation(1);
                                }
                                Err(_) if resume_on_error => {
                                    statistics.add_failed_annotations(1);
                                }
                                Err(err) => error_sender
                                    .send(anyhow::Error::msg(err))
                                    .expect("Could not send error"),
                            }
                        }
                    }

                    Err(_) if resume_on_error => {
                        statistics.add_failed_uploads(1);
                    }

                    Err(err) => error_sender.send(err).expect("Could not send error"),
                }
            })
        }
    });

    drop(error_sender);

    if let Ok(error) = error_receiver.try_recv() {
        return Err(anyhow!("Failed to upload document {}", error));
    };

    Ok(())
}

pub fn run(args: &UploadPackageArgs, client: &Client, pool: &mut Pool) -> Result<()> {
    let UploadPackageArgs {
        file,
        resume_on_error,
        max_attachment_memory_mb,
        dataset_creation_timeout,
        new_project_name,
    } = args;

    let mut package =
        Package::new(file).context("Failed to read package, check path is correct.")?;

    let statistics = Arc::new(Statistics::new());
    let total_documents = package.get_document_count();

    let _progress_bar = get_progress_bar(total_documents as u64, &statistics);

    for dataset in package
        .datasets()
        .context("Could not get package datastes")?
    {
        let packaged_sources = get_ixp_source(&dataset, SourceProvider::Packaged(&mut package))
            .context("Could not get ixp source from package")?;

        let new_dataset = create_dataset(
            new_project_name.clone().unwrap_or(dataset.name),
            dataset.label_defs,
            client,
            *dataset_creation_timeout,
        )
        .context("Could not create dataset")?;

        let new_sources = get_ixp_source(&new_dataset, SourceProvider::Remote(client))
            .context("Could not get ixp source via api")?;

        for (packaged, new) in [
            (&packaged_sources.design_time, &new_sources.design_time),
            (&packaged_sources.runtime, &new_sources.runtime),
        ] {
            unpack_ixp_source(
                packaged,
                new,
                &new_dataset,
                &mut package,
                client,
                pool,
                *resume_on_error,
                &statistics,
                max_attachment_memory_mb,
            )
            .context("Failed to unpack ixp source")?;
        }
    }

    log::info!("Package uploaded.");
    Ok(())
}

#[derive(Debug)]
pub struct Statistics {
    document_reads: AtomicUsize,
    document_uploads: AtomicUsize,
    annotations: AtomicUsize,
    failed_annotations: AtomicUsize,
    failed_reads: AtomicUsize,
    failed_uploads: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            annotations: AtomicUsize::new(0),
            document_uploads: AtomicUsize::new(0),
            document_reads: AtomicUsize::new(0),
            failed_annotations: AtomicUsize::new(0),
            failed_reads: AtomicUsize::new(0),
            failed_uploads: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_annotation(&self, num: usize) {
        self.annotations.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_failed_annotations(&self, num: usize) {
        self.failed_annotations.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_failed_uploads(&self, num: usize) {
        self.failed_uploads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_document_uploads(&self, num: usize) {
        self.document_uploads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_document_reads(&self, num: usize) {
        self.document_reads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn num_document_uploads(&self) -> usize {
        self.document_uploads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_document_reads(&self) -> usize {
        self.document_reads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_annotations(&self) -> usize {
        self.annotations.load(Ordering::SeqCst)
    }

    #[inline]
    fn add_failed_document_reads(&self, num: usize) {
        self.failed_reads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn num_failed_annotations(&self) -> usize {
        self.failed_annotations.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_reads(&self) -> usize {
        self.failed_reads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_uploads(&self) -> usize {
        self.failed_uploads.load(Ordering::SeqCst)
    }
}

fn get_progress_bar(total_comments: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let num_docs_read = statistics.num_document_reads();
            let num_docs_uploaded = statistics.num_document_uploads();
            let num_annotations = statistics.num_annotations();
            let num_failed_annotations = statistics.num_failed_annotations();
            let num_failed_reads = statistics.num_failed_reads();
            let num_failed_uploads = statistics.num_failed_uploads();

            fn get_failed_part(val: usize, name: &str) -> String {
                if val > 0 {
                    format!(" {} {}", val.to_string().bold(), name.dimmed())
                } else {
                    String::new()
                }
            }

            (
                ((num_docs_uploaded + num_docs_read) / 2) as u64,
                format!(
                    "{} {} {} {} {} {}{}{}{}",
                    num_docs_read.to_string().bold(),
                    "docs read".dimmed(),
                    num_docs_uploaded.to_string().bold(),
                    "docs uploaded".dimmed(),
                    num_annotations.to_string().bold(),
                    "annotations".dimmed(),
                    get_failed_part(num_failed_reads, "failed reads"),
                    get_failed_part(num_failed_uploads, "failed uploads"),
                    get_failed_part(num_failed_annotations, "failed annotations")
                ),
            )
        },
        statistics,
        Some(total_comments),
        crate::progress::Options { bytes_units: false },
    )
}
