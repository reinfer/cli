use crate::progress::{Options as ProgressOptions, Progress};
use anyhow::{Context, Result};
use colored::Colorize;
use log::info;
use reinfer_client::{
    resources::comment::{should_skip_serializing_optional_vec, EitherLabelling, HasAnnotations},
    Client, CommentId, CommentUid, DatasetFullName, DatasetIdentifier, NewEntities, NewLabelling,
    NewMoonForm, Source, SourceIdentifier,
};
use serde::{Deserialize, Serialize};
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

    #[structopt(long)]
    /// Whether to use the moon_forms field when creating annotations
    /// for a comment.
    use_moon_forms: bool,
}

pub async fn create(client: &Client, args: &CreateAnnotationsArgs) -> Result<()> {
    let source = client
        .get_source(args.source.clone())
        .await
        .with_context(|| format!("Unable to get source {}", args.source))?;
    let source_name = source.full_name();

    let dataset = client
        .get_dataset(args.dataset.clone())
        .await
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
                args.use_moon_forms,
            )
            .await?;
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
                args.use_moon_forms,
            )
            .await?;
            statistics
        }
    };

    info!(
        "Successfully uploaded {} annotations.",
        statistics.num_annotations(),
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn upload_annotations_from_reader(
    client: &Client,
    source: &Source,
    annotations: impl BufRead,
    statistics: &Statistics,
    dataset_name: &DatasetFullName,
    use_moon_forms: bool,
) -> Result<()> {
    for read_comment_result in read_annotations_iter(annotations, Some(statistics)) {
        let new_comment = read_comment_result?;
        if new_comment.has_annotations() {
            let comment_uid = CommentUid(format!("{}.{}", source.id.0, new_comment.comment.id.0));
            let new_labellings = new_comment.labelling.map(Into::<Vec<NewLabelling>>::into);
            (if !use_moon_forms {
                client.update_labelling(
                    dataset_name,
                    &comment_uid,
                    new_labellings.as_deref(),
                    new_comment.entities.as_ref(),
                    None,
                )
            } else {
                client.update_labelling(
                    dataset_name,
                    &comment_uid,
                    None,
                    None,
                    new_comment.moon_forms.as_deref(),
                )
            })
            .await
            .with_context(|| {
                format!(
                    "Could not update labelling for comment `{}`",
                    &comment_uid.0
                )
            })?;
            statistics.add_annotation();
        }
    }

    Ok(())
}

/// This struct only contains the minimal amount of data required to be able to upload annotations
/// via the api, while still matching the structure of the NewComment struct.  This makes the jsonl
/// files downloaded via `re` compatible with the `re create annotations` command.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct CommentIdComment {
    id: CommentId,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct NewAnnotation {
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
            .with_context(|| format!("Could not read line {} from input stream", line_number));

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
                format!(
                    "Could not parse annotations at line {} from input stream",
                    line_number,
                )
            }),
        )
    })
}

#[derive(Debug)]
pub struct Statistics {
    bytes_read: AtomicUsize,
    annotations: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            bytes_read: AtomicUsize::new(0),
            annotations: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_bytes_read(&self, bytes_read: usize) {
        self.bytes_read.fetch_add(bytes_read, Ordering::SeqCst);
    }

    #[inline]
    fn add_annotation(&self) {
        self.annotations.fetch_add(1, Ordering::SeqCst);
    }

    #[inline]
    fn bytes_read(&self) -> usize {
        self.bytes_read.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn num_annotations(&self) -> usize {
        self.annotations.load(Ordering::SeqCst)
    }
}

fn basic_statistics(statistics: &Statistics) -> (u64, String) {
    let bytes_read = statistics.bytes_read();
    let num_annotations = statistics.num_annotations();
    (
        bytes_read as u64,
        format!("{} {}", num_annotations, "annotations".dimmed(),),
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
