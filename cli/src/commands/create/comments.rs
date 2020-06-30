use colored::Colorize;
use failchain::ResultExt;
use log::info;
use reinfer_client::{
    Client, CommentUid, DatasetFullName, DatasetIdentifier, NewAnnotatedComment, Source,
    SourceIdentifier,
};
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
    errors::{ErrorKind, Result},
    progress::{Options as ProgressOptions, Progress},
};

#[derive(Debug, StructOpt)]
pub struct CreateCommentsArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with comments. If not specified, stdin will be used.
    comments_path: Option<PathBuf>,

    #[structopt(short = "s", long = "source")]
    /// Name or id of the source where the comments will be uploaded.
    source: SourceIdentifier,

    #[structopt(short = "d", long = "dataset")]
    /// Optionally, a dataset (name or id) where to push the annotations. The
    /// dataset must contain the source.
    dataset: Option<DatasetIdentifier>,

    #[structopt(long = "batch-size", default_value = "128")]
    /// Number of comments to batch in a single request.
    batch_size: usize,

    #[structopt(long = "progress")]
    /// Whether to display a progress bar. Only applicable when a file is used.
    progress: Option<bool>,
}

pub fn create(client: &Client, args: &CreateCommentsArgs) -> Result<()> {
    let source = client
        .get_source(args.source.clone())
        .chain_err(|| ErrorKind::Client(format!("Unable to get source {}", args.source)))?;
    let source_name = source.full_name();

    let dataset_name = match args.dataset.as_ref() {
        Some(dataset_ident) => Some(
            client
                .get_dataset(dataset_ident.clone())
                .chain_err(|| ErrorKind::Client(format!("Unable to get dataset {}", args.source)))?
                .full_name(),
        ),
        None => None,
    };

    let statistics = match args.comments_path {
        Some(ref comments_path) => {
            info!(
                "Uploading comments from file `{}` to source `{}` [id: {}]",
                comments_path.display(),
                source_name.0,
                source.id.0,
            );
            let file_metadata = fs::metadata(&comments_path).chain_err(|| {
                ErrorKind::Config(format!(
                    "Could not get file metadata for `{}`",
                    comments_path.display()
                ))
            })?;
            let file = BufReader::new(File::open(comments_path).chain_err(|| {
                ErrorKind::Config(format!("Could not open file `{}`", comments_path.display()))
            })?);
            let statistics = Arc::new(Statistics::new());
            let progress = if args.progress.unwrap_or(true) {
                Some(progress_bar(file_metadata.len(), &statistics))
            } else {
                None
            };
            upload_comments_from_reader(
                &client,
                &source,
                file,
                args.batch_size,
                &statistics,
                dataset_name,
            )?;
            if let Some(mut progress) = progress {
                progress.done();
            }
            Arc::try_unwrap(statistics).unwrap()
        }
        None => {
            info!(
                "Uploading comments from stdin to source `{}` [id: {}]",
                source_name.0, source.id.0,
            );
            let statistics = Statistics::new();
            upload_comments_from_reader(
                &client,
                &source,
                BufReader::new(io::stdin()),
                args.batch_size,
                &statistics,
                dataset_name,
            )?;
            statistics
        }
    };

    info!(
        concat!(
            "Successfully uploaded {} comments [{} new | {} updated | {} unchanged] ",
            "of which {} are annotated."
        ),
        statistics.num_uploaded(),
        statistics.num_new(),
        statistics.num_updated(),
        statistics.num_unchanged(),
        statistics.num_annotations(),
    );

    Ok(())
}

fn upload_comments_from_reader(
    client: &Client,
    source: &Source,
    mut comments: impl BufRead,
    batch_size: usize,
    statistics: &Statistics,
    dataset_name: Option<DatasetFullName>,
) -> Result<()> {
    assert!(batch_size > 0);
    let mut line_number = 1;
    let mut line = String::new();
    let mut batch = Vec::with_capacity(batch_size);
    let mut eof = false;
    let mut with_annotations = Vec::new();
    while !eof {
        line.clear();
        let bytes_read = comments.read_line(&mut line).chain_err(|| {
            ErrorKind::Unknown(format!(
                "Could not read line {} from input stream",
                line_number
            ))
        })?;

        if bytes_read == 0 {
            eof = true;
        } else {
            statistics.add_bytes_read(bytes_read);
            let new_comment = serde_json::from_str::<NewAnnotatedComment>(line.trim_end())
                .chain_err(|| {
                    ErrorKind::Unknown(format!(
                        "Could not parse comment at line {} from input stream",
                        line_number,
                    ))
                })?;

            if dataset_name.is_some() && new_comment.has_annotations() {
                with_annotations.push((
                    CommentUid(format!("{}.{}", source.id.0, new_comment.comment.id.0)),
                    new_comment.clone(),
                ));
            }
            batch.push(new_comment.comment);
        }

        if batch.len() == batch_size || (!batch.is_empty() && eof) {
            // Upload comments
            let result = client
                .sync_comments(&source.full_name(), &batch)
                .chain_err(|| ErrorKind::Client("Could not upload batch of comments".into()))?;
            let num_uploaded = batch.len();
            statistics.add_comments(StatisticsUpdate {
                uploaded: num_uploaded,
                new: result.new,
                updated: result.updated,
                unchanged: result.unchanged,
            });
            batch.clear();

            // Upload annotations
            if let Some(dataset_name) = dataset_name.as_ref() {
                for (comment_uid, new_comment) in with_annotations.iter() {
                    client
                        .update_labelling(
                            &dataset_name,
                            comment_uid,
                            new_comment.labelling.as_ref(),
                            new_comment.entities.as_ref(),
                        )
                        .chain_err(|| {
                            ErrorKind::Client("Could not update labelling for comment".into())
                        })?;
                    statistics.add_annotation();
                }
                with_annotations.clear();
            }
        }

        line_number += 1;
    }

    Ok(())
}

#[derive(Debug)]
pub struct StatisticsUpdate {
    uploaded: usize,
    new: usize,
    updated: usize,
    unchanged: usize,
}

#[derive(Debug)]
pub struct Statistics {
    bytes_read: AtomicUsize,
    uploaded: AtomicUsize,
    new: AtomicUsize,
    updated: AtomicUsize,
    unchanged: AtomicUsize,
    annotations: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            bytes_read: AtomicUsize::new(0),
            uploaded: AtomicUsize::new(0),
            new: AtomicUsize::new(0),
            updated: AtomicUsize::new(0),
            unchanged: AtomicUsize::new(0),
            annotations: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_bytes_read(&self, bytes_read: usize) {
        self.bytes_read.fetch_add(bytes_read, Ordering::SeqCst);
    }

    #[inline]
    fn add_comments(&self, update: StatisticsUpdate) {
        self.uploaded.fetch_add(update.uploaded, Ordering::SeqCst);
        self.new.fetch_add(update.new, Ordering::SeqCst);
        self.updated.fetch_add(update.updated, Ordering::SeqCst);
        self.unchanged.fetch_add(update.unchanged, Ordering::SeqCst);
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
    fn num_uploaded(&self) -> usize {
        self.uploaded.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_new(&self) -> usize {
        self.new.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_updated(&self) -> usize {
        self.updated.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_unchanged(&self) -> usize {
        self.unchanged.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_annotations(&self) -> usize {
        self.annotations.load(Ordering::SeqCst)
    }
}

fn progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let bytes_read = statistics.bytes_read();
            let num_uploaded = statistics.num_uploaded();
            let num_new = statistics.num_new();
            let num_updated = statistics.num_updated();
            let num_unchanged = statistics.num_unchanged();
            let num_annotations = statistics.num_annotations();
            (
                bytes_read as u64,
                format!(
                    "{} {}: {} {} {} {} {} {} [{} {}]",
                    num_uploaded.to_string().bold(),
                    "comments".dimmed(),
                    num_new,
                    "new".dimmed(),
                    num_updated,
                    "upd".dimmed(),
                    num_unchanged,
                    "nop".dimmed(),
                    num_annotations,
                    "annotations".dimmed(),
                ),
            )
        },
        &statistics,
        total_bytes,
        ProgressOptions {
            bytes_units: true,
            ..Default::default()
        },
    )
}
