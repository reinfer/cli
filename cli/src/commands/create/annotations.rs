use crate::progress::{Options as ProgressOptions, Progress};
use anyhow::{Context, Result};
use colored::Colorize;
use log::info;
use reinfer_client::{
    resources::comment::{should_skip_serializing_optional_vec, EitherLabelling, HasAnnotations},
    Client, CommentId, CommentUid, DatasetFullName, DatasetIdentifier, NewEntities, NewLabelling,
    NewMoonForm, Source, SourceIdentifier,
};
use scoped_threadpool::Pool;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::channel;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

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

pub fn create(client: &Client, args: &CreateAnnotationsArgs, pool: &mut Pool) -> Result<()> {
    let source = client
        .get_source(args.source.clone())
        .with_context(|| format!("Unable to get source {}", args.source))?;
    let source_name = source.full_name();

    let dataset = client
        .get_dataset(args.dataset.clone())
        .with_context(|| format!("Unable to get dataset {}", args.dataset))?;
    let dataset_name = dataset.full_name();

    let statistics = match &args.annotations_path {
        Some(annotations_path) => {
            info!(
                "Uploading comments from file `{}` to source `{}` [id: {}] and dataset `{}` [id: {}]",
                annotations_path.display(),
                source_name.0,
                source.id.0,
                dataset_name.0,
                dataset.id.0,
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
                client,
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
                source_name.0, source.id.0, dataset_name.0, dataset.id.0
            );
            let statistics = Statistics::new();
            upload_annotations_from_reader(
                client,
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

pub trait AnnotationStatistic {
    fn add_annotation(&self);
    fn add_failed_annotation(&self);
}

pub trait AttachmentStatistic {
    fn add_attachment(&self);
    fn add_failed_attachment(&self);
}

#[allow(clippy::too_many_arguments)]
pub fn upload_batch_of_annotations(
    annotations_to_upload: &mut Vec<NewAnnotation>,
    client: &Client,
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
                let comment_uid =
                    CommentUid(format!("{}.{}", source.id.0, new_comment.comment.id.0));

                let result = (if new_comment.moon_forms.is_none() {
                    client.update_labelling(
                        dataset_name,
                        &comment_uid,
                        new_comment
                            .labelling
                            .clone()
                            .map(Into::<Vec<NewLabelling>>::into)
                            .as_deref(),
                        new_comment.entities.as_ref(),
                        None,
                    )
                } else {
                    client.update_labelling(
                        dataset_name,
                        &comment_uid,
                        None,
                        new_comment.entities.as_ref(),
                        new_comment.moon_forms.as_deref(),
                    )
                })
                .with_context(|| {
                    format!(
                        "Could not update labelling for comment `{}`",
                        &comment_uid.0
                    )
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

#[allow(clippy::too_many_arguments)]
fn upload_annotations_from_reader(
    client: &Client,
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
                    client,
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
            client,
            source,
            statistics,
            dataset_name,
            pool,
            resume_on_error,
        )?;
    }

    Ok(())
}

/// This struct only contains the minimal amount of data required to be able to upload annotations
/// via the api, while still matching the structure of the NewComment struct.  This makes the jsonl
/// files downloaded via `re` compatible with the `re create annotations` command.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CommentIdComment {
    pub id: CommentId,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NewAnnotation {
    pub comment: CommentIdComment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labelling: Option<EitherLabelling>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<NewEntities>,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec", default)]
    pub moon_forms: Option<Vec<NewMoonForm>>,
}

impl HasAnnotations for NewAnnotation {
    fn has_annotations(&self) -> bool {
        self.labelling.has_annotations()
            || self.entities.has_annotations()
            || self.moon_forms.has_annotations()
    }
}

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

fn progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        basic_statistics,
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: true },
    )
}
