use crate::progress::{Options as ProgressOptions, Progress};
use anyhow::{anyhow, ensure, Context, Result};
use colored::Colorize;
use log::{debug, info};
use reinfer_client::{
    Client, CommentId, CommentUid, DatasetFullName, DatasetIdentifier, NewAnnotatedComment,
    NewComment, NewEntities, NewLabelling, Source, SourceIdentifier,
};
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

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

    #[structopt(long)]
    /// Don't display a progress bar (only applicable when --file is used).
    no_progress: bool,

    #[structopt(long)]
    /// Whether to allow duplicate comment IDs in the input.
    allow_duplicates: bool,

    #[structopt(long)]
    /// Whether to allow overwriting existing comments in the source.
    ///
    /// If not set, the upload will halt when encountering a comment with an ID
    /// which is already associated to different data on the platform.
    overwrite: bool,
}

pub fn create(client: &Client, args: &CreateCommentsArgs) -> Result<()> {
    let source = client
        .get_source(args.source.clone())
        .with_context(|| format!("Unable to get source {}", args.source))?;
    let source_name = source.full_name();

    let dataset_name = match args.dataset.as_ref() {
        Some(dataset_ident) => Some(
            client
                .get_dataset(dataset_ident.clone())
                .with_context(|| format!("Unable to get dataset {}", args.source))?
                .full_name(),
        ),
        None => None,
    };

    ensure!(args.batch_size > 0, "--batch-size must be greater than 0");

    let statistics = match args.comments_path {
        Some(ref comments_path) => {
            info!(
                "Uploading comments from file `{}` to source `{}` [id: {}]",
                comments_path.display(),
                source_name.0,
                source.id.0,
            );
            let mut file =
                BufReader::new(File::open(comments_path).with_context(|| {
                    format!("Could not open file `{}`", comments_path.display())
                })?);
            let file_metadata = file.get_ref().metadata().with_context(|| {
                format!(
                    "Could not get file metadata for `{}`",
                    comments_path.display()
                )
            })?;

            if !args.allow_duplicates {
                debug!(
                    "Checking `{}` for duplicate comment ids",
                    comments_path.display(),
                );
                check_no_duplicate_ids(&mut file)?;

                file.seek(SeekFrom::Start(0)).with_context(|| {
                    "Unable to seek to file start after checking for duplicate ids"
                })?;
            }

            let statistics = Arc::new(Statistics::new());
            let progress = if args.no_progress {
                None
            } else {
                Some(progress_bar(
                    file_metadata.len(),
                    &statistics,
                    args.overwrite,
                ))
            };
            upload_comments_from_reader(
                &client,
                &source,
                file,
                args.batch_size,
                &statistics,
                dataset_name.as_ref(),
                args.overwrite,
                args.allow_duplicates,
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
            ensure!(
                args.allow_duplicates,
                "--allow-duplicates is required when uploading from stdin"
            );
            let statistics = Statistics::new();
            upload_comments_from_reader(
                &client,
                &source,
                BufReader::new(io::stdin()),
                args.batch_size,
                &statistics,
                dataset_name.as_ref(),
                args.overwrite,
                args.allow_duplicates,
            )?;
            statistics
        }
    };

    if args.overwrite {
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
    } else {
        // PUT comments endpoint doesn't return statistics, so can't give the breakdown when not
        // forcing everything the sync endpoint (via --overwrite)
        info!(
            "Successfully uploaded {} comments (of which {} are annotated).",
            statistics.num_uploaded(),
            statistics.num_annotations(),
        );
    }

    Ok(())
}

fn read_comments_iter<'a>(
    mut comments: impl BufRead + 'a,
    statistics: Option<&'a Statistics>,
) -> impl Iterator<Item = Result<NewAnnotatedComment>> + 'a {
    let mut line = String::new();
    let mut line_number: u32 = 0;
    std::iter::from_fn(move || {
        line_number += 1;
        line.clear();

        let read_result = comments
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
            serde_json::from_str::<NewAnnotatedComment>(line.trim_end()).with_context(|| {
                format!(
                    "Could not parse comment at line {} from input stream",
                    line_number,
                )
            }),
        )
    })
}

fn check_no_duplicate_ids(comments: impl BufRead) -> Result<()> {
    let mut seen = HashSet::new();
    for read_comment_result in read_comments_iter(comments, None) {
        let new_comment = read_comment_result?;
        let id = new_comment.comment.id;

        if !seen.insert(id.clone()) {
            return Err(anyhow!("Duplicate comments with id {}", id.0));
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn upload_batch(
    client: &Client,
    source: &Source,
    dataset_name: Option<&DatasetFullName>,
    statistics: &Statistics,
    comments_to_put: &[NewComment],
    comments_to_sync: &[NewComment],
    annotations: &[(CommentUid, Option<NewLabelling>, Option<NewEntities>)],
    audio_paths: &[(CommentId, PathBuf)],
) -> Result<()> {
    let mut uploaded = 0;
    let mut new = 0;
    let mut updated = 0;
    let mut unchanged = 0;

    // Upload comments
    if !comments_to_put.is_empty() {
        client
            .put_comments(&source.full_name(), &comments_to_put)
            .context("Could not put batch of comments")?;

        uploaded += comments_to_put.len();
    }

    if !comments_to_sync.is_empty() {
        let result = client
            .sync_comments(&source.full_name(), &comments_to_sync)
            .context("Could not sync batch of comments")?;

        uploaded += comments_to_sync.len();
        new += result.new;
        updated += result.updated;
        unchanged += result.unchanged;
    }

    statistics.add_comments(StatisticsUpdate {
        uploaded,
        new,
        updated,
        unchanged,
    });

    // Upload annotations
    if let Some(dataset_name) = dataset_name.as_ref() {
        for (comment_uid, labelling, entities) in annotations.iter() {
            client
                .update_labelling(
                    &dataset_name,
                    comment_uid,
                    labelling.as_ref(),
                    entities.as_ref(),
                )
                .context("Could not update labelling for comment")?;
            statistics.add_annotation();
        }
    }

    // Upload audio
    for (comment_id, audio_path) in audio_paths.iter() {
        client
            .put_comment_audio(&source.id, comment_id, audio_path)
            .with_context(|| {
                format!(
                    "Could not upload audio file at `{}` for comment id `{}",
                    audio_path.display(),
                    comment_id.0,
                )
            })?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn upload_comments_from_reader(
    client: &Client,
    source: &Source,
    comments: impl BufRead,
    batch_size: usize,
    statistics: &Statistics,
    dataset_name: Option<&DatasetFullName>,
    overwrite: bool,
    allow_duplicates: bool,
) -> Result<()> {
    assert!(batch_size > 0);

    let mut comments_to_put = Vec::with_capacity(batch_size);
    let mut comments_to_sync = Vec::new();
    let mut annotations = Vec::new();
    let mut audio_paths = Vec::new();

    // if --overwrite, everything will go to comments_to_sync, so put the default capacity there.
    if overwrite {
        std::mem::swap(&mut comments_to_put, &mut comments_to_sync)
    }

    let mut should_sync_comment = {
        let mut seen = HashSet::new();
        move |id: &CommentId| overwrite || (allow_duplicates && !seen.insert(id.clone()))
    };

    for read_comment_result in read_comments_iter(comments, Some(statistics)) {
        let new_comment = read_comment_result?;

        if dataset_name.is_some() && new_comment.has_annotations() {
            annotations.push((
                CommentUid(format!("{}.{}", source.id.0, new_comment.comment.id.0)),
                new_comment.labelling,
                new_comment.entities,
            ));
        }

        if let Some(audio_path) = new_comment.audio_path {
            audio_paths.push((new_comment.comment.id.clone(), audio_path));
        }

        if should_sync_comment(&new_comment.comment.id) {
            comments_to_sync.push(new_comment.comment);
        } else {
            comments_to_put.push(new_comment.comment);
        }

        if (comments_to_put.len() + comments_to_sync.len()) >= batch_size {
            upload_batch(
                client,
                source,
                dataset_name,
                statistics,
                &comments_to_put,
                &comments_to_sync,
                &annotations,
                &audio_paths,
            )?;
            comments_to_put.clear();
            comments_to_sync.clear();
            annotations.clear();
            audio_paths.clear();
        }
    }

    if !comments_to_put.is_empty() || !comments_to_sync.is_empty() {
        upload_batch(
            client,
            source,
            dataset_name,
            statistics,
            &comments_to_put,
            &comments_to_sync,
            &annotations,
            &audio_paths,
        )?;
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

/// Detailed statistics - only make sense if using --overwrite (i.e. exclusively sync endpoint)
/// as the PUT endpoint doesn't return any statistics.
fn detailed_statistics(statistics: &Statistics) -> (u64, String) {
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
}

/// Basic statistics always applicable (just counts of quantities sent to the platform).
fn basic_statistics(statistics: &Statistics) -> (u64, String) {
    let bytes_read = statistics.bytes_read();
    let num_uploaded = statistics.num_uploaded();
    let num_annotations = statistics.num_annotations();
    (
        bytes_read as u64,
        format!(
            "{} {} [{} {}]",
            num_uploaded.to_string().bold(),
            "comments".dimmed(),
            num_annotations,
            "annotations".dimmed(),
        ),
    )
}

fn progress_bar(
    total_bytes: u64,
    statistics: &Arc<Statistics>,
    use_detailed_statistics: bool,
) -> Progress {
    Progress::new(
        if use_detailed_statistics {
            detailed_statistics
        } else {
            basic_statistics
        },
        &statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: true },
    )
}

#[cfg(test)]
mod tests {
    use super::{check_no_duplicate_ids, read_comments_iter, Statistics};
    use std::io::{BufReader, Cursor};

    const SAMPLE_DUPLICATES: &str = include_str!("../../../tests/samples/duplicates.jsonl");

    #[test]
    fn test_read_comments_iter() {
        let reader = BufReader::new(Cursor::new(SAMPLE_DUPLICATES));
        let statistics = Statistics::new();

        let comments: Vec<_> = read_comments_iter(reader, Some(&statistics)).collect();

        assert_eq!(comments.len(), 5);
        assert_eq!(statistics.bytes_read(), SAMPLE_DUPLICATES.len());
    }

    #[test]
    fn check_detects_duplicates() {
        let reader = BufReader::new(Cursor::new(SAMPLE_DUPLICATES));
        let result = check_no_duplicate_ids(reader);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Duplicate comments with id"));
    }
}
