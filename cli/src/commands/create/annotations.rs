//! Commands for creating and uploading comment annotations

// Standard library imports
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::channel,
        Arc,
    },
};

// External crate imports
use anyhow::{Context, Result};
use colored::Colorize;
use log::info;
use scoped_threadpool::Pool;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{configuration::Configuration, datasets_api::update_comment_labelling},
    models::{
        EntitiesNew, GroupLabellingsRequest, MoonFormGroupUpdate, Source,
        UpdateCommentLabellingRequest,
    },
};

// Local crate imports
use crate::{
    progress::{Options as ProgressOptions, Progress},
    utils::{
        conversions::{should_skip_serializing_optional_vec, HasAnnotations},
        resolve_dataset, resolve_source,
        types::identifiers::{DatasetIdentifier, SourceIdentifier},
        DatasetFullName,
    },
};

/// Command line arguments for creating annotations
#[derive(Debug, StructOpt)]
pub struct CreateAnnotationsArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with annotations. If not specified, stdin will be used.
    annotations_path: Option<PathBuf>,

    #[structopt(short = "s", long = "source")]
    /// Name or id of the source containing the annotated comments
    source: SourceIdentifier,

    #[structopt(short = "d", long = "dataset")]
    /// Dataset (name or id) where to push the annotations. The dataset must contain the source.
    dataset: DatasetIdentifier,

    #[structopt(long)]
    /// Don't display a progress bar (only applicable when --file is used).
    no_progress: bool,

    #[structopt(long = "batch-size", default_value = "128")]
    /// Number of comments to batch in a single request.
    batch_size: usize,

    #[structopt(long = "resume-on-error")]
    /// Whether to attempt to resume processing on error
    resume_on_error: bool,
}

/// Create annotations from file or stdin and upload them to a dataset
pub fn create(config: &Configuration, args: &CreateAnnotationsArgs, pool: &mut Pool) -> Result<()> {
    let source = resolve_source(config, &args.source)?;
    let source_name = DatasetFullName::from_str(&format!("{}/{}", source.owner, source.name))?;

    let dataset = resolve_dataset(config, &args.dataset)?;
    let dataset_name = DatasetFullName::from_str(&format!("{}/{}", dataset.owner, dataset.name))?;

    let statistics = match &args.annotations_path {
        Some(annotations_path) => {
            info!(
                "Uploading comments from file `{}` to source `{}` [id: {}] and dataset `{}` [id: {}]",
                annotations_path.display(),
                source_name.0,
                source.id,
                dataset_name.0,
                dataset.id,
            );
            let file = BufReader::new(File::open(annotations_path).with_context(|| {
                format!("Could not open file `{}`", annotations_path.display())
            })?);
            let file_metadata = file.get_ref().metadata().with_context(|| {
                format!(
                    "Could not get file metadata for `{}`",
                    annotations_path.display()
                )
            })?;

            let statistics = Arc::new(Statistics::new());
            let progress = if args.no_progress {
                None
            } else {
                Some(progress_bar(file_metadata.len(), &statistics))
            };
            upload_annotations_from_reader(
                config,
                &source,
                file,
                &statistics,
                &dataset_name,
                args.batch_size,
                pool,
                args.resume_on_error,
            )?;
            if let Some(mut progress) = progress {
                progress.done();
            }
            Arc::try_unwrap(statistics)
                .expect("Not all references to `statistics` have been disposed of")
        }
        None => {
            info!(
                "Uploading annotations from stdin to source `{}` [id: {}] and dataset `{} [id: {}]",
                source_name.0, source.id, dataset_name.0, dataset.id
            );
            let statistics = Statistics::new();
            upload_annotations_from_reader(
                config,
                &source,
                BufReader::new(io::stdin()),
                &statistics,
                &dataset_name,
                args.batch_size,
                pool,
                args.resume_on_error,
            )?;
            statistics
        }
    };

    info!(
        "Successfully uploaded {} annotations.",
        statistics.num_annotations(),
    );
    Ok(())
}

// ============================================================================
// Traits
// ============================================================================

/// Trait for tracking annotation upload statistics
pub trait AnnotationStatistic {
    fn add_annotation(&self);
    fn add_failed_annotation(&self);
}

/// Trait for tracking attachment upload statistics
pub trait AttachmentStatistic {
    fn add_attachment(&self);
    fn add_failed_attachment(&self);
}

// ============================================================================
// Batch Upload Functions
// ============================================================================

/// Upload a batch of annotations to a dataset
#[allow(clippy::too_many_arguments)]
pub fn upload_batch_of_annotations(
    annotations_to_upload: &mut Vec<NewAnnotation>,
    config: &Configuration,
    source: &Source,
    statistics: &(impl AnnotationStatistic + std::marker::Sync),
    dataset_name: &DatasetFullName,
    pool: &mut Pool,
    resume_on_error: bool,
) -> Result<()> {
    let (error_sender, error_receiver) = channel();

    pool.scoped(|scope| {
        annotations_to_upload.iter().for_each(|new_comment| {
            let error_sender = error_sender.clone();

            scope.execute(move || {
                let comment_uid = format!("{}.{}", source.id, new_comment.comment.id);

                let request = UpdateCommentLabellingRequest {
                    context: None,
                    labelling: new_comment.labelling.clone(),
                    entities: new_comment.entities.clone().map(Box::new),
                    moon_forms: new_comment.moon_forms.clone(),
                };

                let result = update_comment_labelling(
                    config,
                    dataset_name.owner(),
                    dataset_name.name(),
                    &comment_uid,
                    request,
                )
                .with_context(|| {
                    format!("Could not update labelling for comment `{}`", &comment_uid)
                });

                if let Err(error) = result {
                    error_sender.send(error).expect("Could not send error");
                    statistics.add_failed_annotation();
                } else {
                    statistics.add_annotation();
                }
            });
        })
    });

    if let Ok(error) = error_receiver.try_recv() {
        if resume_on_error {
            annotations_to_upload.clear();
            Ok(())
        } else {
            Err(error)
        }
    } else {
        annotations_to_upload.clear();
        Ok(())
    }
}

/// Upload annotations from a reader (file or stdin) in batches
#[allow(clippy::too_many_arguments)]
fn upload_annotations_from_reader(
    config: &Configuration,
    source: &Source,
    annotations: impl BufRead,
    statistics: &Statistics,
    dataset_name: &DatasetFullName,
    batch_size: usize,
    pool: &mut Pool,
    resume_on_error: bool,
) -> Result<()> {
    let mut annotations_to_upload = Vec::new();

    for read_comment_result in read_annotations_iter(annotations, Some(statistics)) {
        let new_comment = read_comment_result?;
        if new_comment.has_annotations() {
            annotations_to_upload.push(new_comment);

            if annotations_to_upload.len() >= batch_size {
                upload_batch_of_annotations(
                    &mut annotations_to_upload,
                    config,
                    source,
                    statistics,
                    dataset_name,
                    pool,
                    resume_on_error,
                )?;
            }
        }
    }

    if !annotations_to_upload.is_empty() {
        upload_batch_of_annotations(
            &mut annotations_to_upload,
            config,
            source,
            statistics,
            dataset_name,
            pool,
            resume_on_error,
        )?;
    }

    Ok(())
}

// ============================================================================
// Data Structures
// ============================================================================

/// Minimal comment structure containing only the ID needed for annotation uploads
/// This makes JSONL files downloaded via `re` compatible with the `re create annotations` command
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CommentIdComment {
    pub id: String,
}

/// Structure for annotation data that can be uploaded to the API
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NewAnnotation {
    pub comment: CommentIdComment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labelling: Option<Vec<GroupLabellingsRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<EntitiesNew>,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec", default)]
    pub moon_forms: Option<Vec<MoonFormGroupUpdate>>,
}

impl HasAnnotations for NewAnnotation {
    fn has_annotations(&self) -> bool {
        self.labelling.has_annotations()
            || self.entities.has_annotations()
            || self.moon_forms.has_annotations()
    }
}

/// Create an iterator that reads and parses annotations from a reader
fn read_annotations_iter<'a>(
    mut annotations: impl BufRead + 'a,
    statistics: Option<&'a Statistics>,
) -> impl Iterator<Item = Result<NewAnnotation>> + 'a {
    let mut line = String::new();
    let mut line_number: u32 = 0;
    std::iter::from_fn(move || {
        line_number += 1;
        line.clear();

        let read_result = annotations
            .read_line(&mut line)
            .with_context(|| format!("Could not read line {line_number} from input stream"));

        match read_result {
            Ok(0) => return None,
            Ok(bytes_read) => {
                if let Some(s) = statistics {
                    s.add_bytes_read(bytes_read)
                }
            }
            Err(e) => return Some(Err(e)),
        }

        Some(
            serde_json::from_str::<NewAnnotation>(line.trim_end()).with_context(|| {
                format!("Could not parse annotations at line {line_number} from input stream")
            }),
        )
    })
}

// ============================================================================
// Statistics Tracking
// ============================================================================

/// Thread-safe statistics tracking for annotation uploads
#[derive(Debug)]
pub struct Statistics {
    bytes_read: AtomicUsize,
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
            bytes_read: AtomicUsize::new(0),
            annotations: AtomicUsize::new(0),
            failed_annotations: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_bytes_read(&self, bytes_read: usize) {
        self.bytes_read.fetch_add(bytes_read, Ordering::SeqCst);
    }

    #[inline]
    fn bytes_read(&self) -> usize {
        self.bytes_read.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn num_annotations(&self) -> usize {
        self.annotations.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn num_failed_annotations(&self) -> usize {
        self.failed_annotations.load(Ordering::SeqCst)
    }
}

/// Generate basic progress statistics for display
fn basic_statistics(statistics: &Statistics) -> (u64, String) {
    let bytes_read = statistics.bytes_read();
    let num_annotations = statistics.num_annotations();
    let num_failed_annotations = statistics.num_failed_annotations();

    let failed_annotations_string = if num_failed_annotations > 0 {
        format!(" {num_failed_annotations} {}", "skipped".dimmed())
    } else {
        String::new()
    };

    (
        bytes_read as u64,
        format!(
            "{} {}{}",
            num_annotations,
            "annotations".dimmed(),
            failed_annotations_string
        ),
    )
}

/// Create a progress bar for annotation upload tracking
fn progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        basic_statistics,
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: true },
    )
}
