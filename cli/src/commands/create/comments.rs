//! Commands for creating comments with annotations and audio support
//!
//! This module provides functionality to:
//! - Upload comments from files or stdin
//! - Handle comment annotations
//! - Process audio attachments
//! - Track upload statistics and progress

// Standard library imports
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader, Seek},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

// External crate imports
use anyhow::{anyhow, ensure, Context, Result};
use colored::Colorize;
use log::{debug, info};
use scoped_threadpool::Pool;
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{
        attachments_api::upload_comment_attachment,
        comments_api::{add_comments, sync_comments},
        configuration::Configuration,
    },
    models,
};

// Local crate imports
use crate::{
    commands::{
        create::annotations::{upload_batch_of_annotations, AnnotationStatistic, NewAnnotation},
        ensure_uip_user_consents_to_ai_unit_charge, LocalAttachmentPath,
    },
    progress::{Options as ProgressOptions, Progress},
    utils::{
        add_comments_with_split_on_failure, handle_split_on_failure_result, resolve_dataset,
        resolve_source, set_comment_audio, sync_comments_with_split_on_failure,
        types::names::FullName, CommentId, DatasetIdentifier, SourceId, SourceIdentifier,
    },
};

use super::annotations::AttachmentStatistic;

// ============================================================================
// Type Definitions
// ============================================================================

/// Structure to parse JSON that contains comment data, annotations, and audio paths
/// This supports the full comment creation workflow including annotations and audio
#[derive(Debug, serde::Deserialize)]
struct CommentWithAnnotationsAndAudio {
    pub comment: models::CommentNew,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labelling: Option<Vec<models::GroupLabellingsRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<models::EntitiesNew>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moon_forms: Option<Vec<models::MoonFormGroupUpdate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_path: Option<PathBuf>,
}

/// Command line arguments for creating comments with support for annotations and attachments
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

    #[structopt(short = "n", long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,

    #[structopt(short = "y", long = "yes")]
    /// Consent to ai unit charge. Suppresses confirmation prompt.
    yes: bool,

    #[structopt(long = "resume-on-error")]
    /// Whether to attempt to resume processing on error
    resume_on_error: bool,

    #[structopt(short = "a", long = "attachments", parse(from_os_str))]
    /// Path to folder containing the attachments to upload
    attachments_dir: Option<PathBuf>,
}

// ============================================================================
// Main Entry Point
// ============================================================================

/// Create comments from file or stdin with full support for annotations and audio
///
/// This function handles:
/// - Comment validation and duplicate checking
/// - Progress tracking for file uploads
/// - Batch processing for efficiency
/// - Annotation uploads to datasets
/// - Audio file uploads
pub fn create(config: &Configuration, args: &CreateCommentsArgs, pool: &mut Pool) -> Result<()> {
    if !args.no_charge && !args.yes {
        ensure_uip_user_consents_to_ai_unit_charge(&config.base_path.parse()?)?;
    }

    ensure!(args.batch_size > 0, "--batch-size must be greater than 0");

    if let Some(attachments_dir) = &args.attachments_dir {
        ensure!(
            attachments_dir.is_dir(),
            "--attachments must be a directory"
        );
        ensure!(
            attachments_dir.exists(),
            "--attachments directory must exist"
        )
    }

    let source = resolve_source(config, &args.source)?;

    let source_name = format!("{}/{}", source.owner, source.name);

    let dataset_name = match args.dataset.as_ref() {
        Some(dataset_id) => {
            let dataset = resolve_dataset(config, dataset_id)?;
            Some(FullName::from_str(&format!(
                "{}/{}",
                dataset.owner, dataset.name
            ))?)
        }
        None => None,
    };

    let statistics = match &args.comments_path {
        Some(comments_path) => {
            info!(
                "Uploading comments from file `{}` to source `{}` [id: {}]",
                comments_path.display(),
                source_name,
                source.id,
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

                file.rewind().with_context(|| {
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
                config,
                &source,
                file,
                args.batch_size,
                &statistics,
                dataset_name.as_ref(),
                args.overwrite,
                args.allow_duplicates,
                args.no_charge,
                pool,
                args.resume_on_error,
                &args.attachments_dir,
            )?;
            if let Some(mut progress) = progress {
                progress.done();
            }
            Arc::try_unwrap(statistics).unwrap()
        }
        None => {
            info!(
                "Uploading comments from stdin to source `{}` [id: {}]",
                source_name, source.id,
            );
            ensure!(
                args.allow_duplicates,
                "--allow-duplicates is required when uploading from stdin"
            );
            let statistics = Statistics::new();
            upload_comments_from_reader(
                config,
                &source,
                BufReader::new(io::stdin()),
                args.batch_size,
                &statistics,
                dataset_name.as_ref(),
                args.overwrite,
                args.allow_duplicates,
                args.no_charge,
                pool,
                args.resume_on_error,
                &args.attachments_dir,
            )?;
            statistics
        }
    };

    if args.overwrite {
        info!(
            concat!(
                "Successfully uploaded {} comments [{} new | {} updated | {} unchanged | {} skipped] ",
                "of which {} are annotated."
            ),
            statistics.num_uploaded(),
            statistics.num_new(),
            statistics.num_updated(),
            statistics.num_unchanged(),
            statistics.num_failed_comments(),
            statistics.num_annotations(),
        );
    } else {
        // PUT comments endpoint doesn't return statistics, so can't give the breakdown when not
        // forcing everything the sync endpoint (via --overwrite)
        info!(
            "Successfully uploaded {} comments (of which {} are annotated). {} skipped",
            statistics.num_uploaded(),
            statistics.num_annotations(),
            statistics.num_failed_comments()
        );
    }

    Ok(())
}

// ============================================================================
// JSON Parsing and Validation
// ============================================================================

/// Create an iterator that parses JSONL comments with annotations and audio paths
///
/// Returns tuples of (comment, annotations, audio_path) for processing
fn read_comments_iter<'a>(
    mut comments: impl BufRead + 'a,
    statistics: Option<&'a Statistics>,
) -> impl Iterator<
    Item = Result<(
        models::CommentNew,
        Option<Vec<NewAnnotation>>,
        Option<PathBuf>,
    )>,
> + 'a {
    let mut line = String::new();
    let mut line_number: u32 = 0;
    std::iter::from_fn(move || {
        line_number += 1;
        line.clear();

        let read_result = comments
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
            serde_json::from_str::<CommentWithAnnotationsAndAudio>(line.trim_end())
                .with_context(|| {
                    format!("Could not parse comment at line {line_number} from input stream")
                })
                .map(|parsed| {
                    let comment = parsed.comment;
                    let labelling = parsed.labelling;
                    let entities = parsed.entities;
                    let moon_forms = parsed.moon_forms;
                    let audio_path = parsed.audio_path;

                    // Extract annotations if present
                    let annotations =
                        if labelling.is_some() || entities.is_some() || moon_forms.is_some() {
                            // Create annotations from the parsed data
                            let mut new_annotations = Vec::new();

                            // Convert to NewAnnotation format for annotations processing
                            let annotation = NewAnnotation {
                                comment: crate::commands::create::annotations::CommentIdComment {
                                    id: comment.id.clone(),
                                },
                                labelling,
                                entities,
                                moon_forms,
                            };
                            new_annotations.push(annotation);
                            Some(new_annotations)
                        } else {
                            None
                        };

                    (comment, annotations, audio_path)
                }),
        )
    })
}

/// Check for duplicate comment IDs in the input stream to prevent data corruption
fn check_no_duplicate_ids(comments: impl BufRead) -> Result<()> {
    let mut seen = HashSet::new();
    for read_comment_result in read_comments_iter(comments, None) {
        let (new_comment, _, _) = read_comment_result?;
        let id = new_comment.id;

        if !seen.insert(id.clone()) {
            return Err(anyhow!("Duplicate comments with id {}", id));
        }
    }

    Ok(())
}

// ============================================================================
// Upload Functions
// ============================================================================

/// Upload a single attachment file for a comment to the API
fn upload_local_attachment(
    comment_id: &CommentId,
    attachment: &mut models::Attachment,
    index: usize,
    config: &Configuration,
    attachments_dir: &Path,
    source_id: &SourceId,
) -> Result<()> {
    let local_attachemnt = LocalAttachmentPath {
        index,
        name: attachment.name.clone(),
        parent_dir: attachments_dir.join(comment_id.as_ref()),
    };

    match upload_comment_attachment(
        config,
        source_id.as_ref(),
        comment_id.as_ref(),
        &index.to_string(),
        Some(local_attachemnt.path()),
    ) {
        Ok(response) => {
            // Update attachment with response data
            attachment.attachment_reference = None;
            attachment.content_hash = Some(response.content_hash);
            Ok(())
        }
        Err(err) => {
            // Clear attachment reference on error
            attachment.attachment_reference = None;
            Err(anyhow::Error::msg(err))
        }
    }
}

/// Upload all attachments for a batch of comments with error handling
fn upload_attachments_for_comments(
    config: &Configuration,
    comments: &mut [models::CommentNew],
    attachments_dir: &Path,
    statistics: &Statistics,
    source_id: &SourceId,
    resume_on_error: bool,
) -> Result<()> {
    for comment in comments.iter_mut() {
        let comment_id = CommentId::from_str(&comment.id)?;
        if let Some(attachments) = &mut comment.attachments {
            for (index, attachment) in attachments.iter_mut().enumerate() {
                match upload_local_attachment(
                    &comment_id,
                    attachment,
                    index,
                    config,
                    attachments_dir,
                    source_id,
                ) {
                    Ok(_) => {
                        statistics.add_attachment();
                    }
                    Err(err) => {
                        if resume_on_error {
                            statistics.add_failed_attachment();
                        } else {
                            return Err(err);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Upload a batch of comments with their attachments and audio files
///
/// This function handles both PUT (new comments) and SYNC (update existing) operations
/// based on the batch configuration
#[allow(clippy::too_many_arguments)]
fn upload_batch_of_comments(
    config: &Configuration,
    source: &models::Source,
    statistics: &Statistics,
    comments_to_put: &mut Vec<models::CommentNew>,
    comments_to_sync: &mut Vec<models::CommentNew>,
    audio_paths: &mut Vec<(String, PathBuf)>,
    no_charge: bool,
    attachments_dir: &Option<PathBuf>,
    resume_on_error: bool,
) -> Result<()> {
    let mut uploaded: usize = 0;
    let mut new: usize = 0;
    let mut updated: usize = 0;
    let mut unchanged: usize = 0;
    let mut failed: usize = 0;

    // Upload comments
    if !comments_to_put.is_empty() {
        if let Some(attachments_dir) = attachments_dir {
            let source_id = SourceId::from_str(&source.id)
                .with_context(|| format!("Invalid source ID: {}", source.id))?;
            upload_attachments_for_comments(
                config,
                comments_to_put,
                attachments_dir,
                statistics,
                &source_id,
                resume_on_error,
            )?;
        }

        if resume_on_error {
            // Use cleaner wrapper function for split-on-failure
            let result = add_comments_with_split_on_failure(
                config,
                &source.owner,
                &source.name,
                comments_to_put.to_vec(),
                Some(no_charge),
            )
            .context("Could not put batch of comments")?;

            failed += result.num_failed;
            handle_split_on_failure_result(result, comments_to_put.len(), "add_comments");
        } else {
            let request = models::AddCommentsRequest::new(comments_to_put.to_vec());
            add_comments(
                config,
                &source.owner,
                &source.name,
                request,
                Some(no_charge),
            )
            .context("Could not put batch of comments")?;
        }
        uploaded += comments_to_put.len() - failed;
    }

    if !comments_to_sync.is_empty() {
        if let Some(attachments_dir) = attachments_dir {
            let source_id = SourceId::from_str(&source.id)
                .with_context(|| format!("Invalid source ID: {}", source.id))?;
            upload_attachments_for_comments(
                config,
                comments_to_sync,
                attachments_dir,
                statistics,
                &source_id,
                resume_on_error,
            )?;
        }
        let result = if resume_on_error {
            // Use cleaner wrapper function for split-on-failure
            let split_result = sync_comments_with_split_on_failure(
                config,
                &source.owner,
                &source.name,
                comments_to_sync.to_vec(),
                Some(no_charge),
            )
            .context("Could not sync batch of comments")?;

            failed += split_result.num_failed;
            handle_split_on_failure_result(
                split_result.clone(),
                comments_to_sync.len(),
                "sync_comments",
            );
            split_result.response
        } else {
            let request = models::SyncCommentsRequest::new(comments_to_sync.to_vec());
            sync_comments(
                config,
                &source.owner,
                &source.name,
                request,
                Some(no_charge),
            )
            .context("Could not sync batch of comments")?
        };

        uploaded += comments_to_sync.len() - failed;
        new += result.new as usize;
        updated += result.updated as usize;
        unchanged += result.unchanged as usize;
    }

    statistics.add_comments(StatisticsUpdate {
        uploaded,
        new,
        updated,
        unchanged,
        failed,
    });

    // Upload audio
    for (comment_id, audio_path) in audio_paths.iter() {
        let source_id = SourceId::from_str(&source.id)
            .with_context(|| format!("Invalid source ID: {}", source.id))?;
        let comment_id = CommentId::from_str(comment_id)
            .with_context(|| format!("Invalid comment ID: {}", comment_id))?;

        set_comment_audio(
            config,
            source_id.as_ref(),
            comment_id.as_ref(),
            audio_path.clone(),
        )
        .with_context(|| {
            format!(
                "Could not upload audio file at `{}` for comment id `{}`",
                audio_path.display(),
                comment_id,
            )
        })?;
    }
    comments_to_put.clear();
    comments_to_sync.clear();
    audio_paths.clear();

    Ok(())
}

/// Main function to process and upload comments from a reader (file or stdin)
///
/// This function handles the complete comment upload workflow:
/// - Parsing comments with annotations and audio paths
/// - Batching for efficient uploads
/// - Managing comment deduplication
/// - Coordinating annotation uploads to datasets
#[allow(clippy::too_many_arguments)]
fn upload_comments_from_reader(
    config: &Configuration,
    source: &models::Source,
    comments: impl BufRead,
    batch_size: usize,
    statistics: &Statistics,
    dataset_name: Option<&FullName>,
    overwrite: bool,
    allow_duplicates: bool,
    no_charge: bool,
    pool: &mut Pool,
    resume_on_error: bool,
    attachments_dir: &Option<PathBuf>,
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
        move |id: &String| overwrite || (allow_duplicates && !seen.insert(id.clone()))
    };

    for read_comment_result in read_comments_iter(comments, Some(statistics)) {
        let (new_comment, comment_annotations, audio_path) = read_comment_result?;

        // Handle annotations
        if let Some(comment_annotations) = comment_annotations {
            for annotation in comment_annotations {
                annotations.push(annotation);
            }
        }

        // Handle audio path
        if let Some(audio_path) = audio_path {
            audio_paths.push((new_comment.id.clone(), audio_path));
        }

        if should_sync_comment(&new_comment.id) {
            comments_to_sync.push(new_comment);
        } else {
            comments_to_put.push(new_comment);
        }

        if (comments_to_put.len() + comments_to_sync.len()) >= batch_size {
            upload_batch_of_comments(
                config,
                source,
                statistics,
                &mut comments_to_put,
                &mut comments_to_sync,
                &mut audio_paths,
                no_charge,
                attachments_dir,
                resume_on_error,
            )?;
        }

        if let Some(dataset_name) = dataset_name {
            if annotations.len() >= batch_size {
                upload_batch_of_comments(
                    config,
                    source,
                    statistics,
                    &mut comments_to_put,
                    &mut comments_to_sync,
                    &mut audio_paths,
                    no_charge,
                    attachments_dir,
                    resume_on_error,
                )?;

                // Upload annotations to dataset if provided
                let dataset_full_name = dataset_name.clone();
                upload_batch_of_annotations(
                    &mut annotations,
                    config,
                    source,
                    statistics,
                    &dataset_full_name,
                    pool,
                    resume_on_error,
                )?;
            }
        }
    }

    if !comments_to_put.is_empty() || !comments_to_sync.is_empty() {
        upload_batch_of_comments(
            config,
            source,
            statistics,
            &mut comments_to_put,
            &mut comments_to_sync,
            &mut audio_paths,
            no_charge,
            attachments_dir,
            resume_on_error,
        )?;
    }

    if let Some(dataset_name) = dataset_name {
        if !annotations.is_empty() {
            // Upload annotations to dataset if provided
            let dataset_full_name = dataset_name.clone();
            upload_batch_of_annotations(
                &mut annotations,
                config,
                source,
                statistics,
                &dataset_full_name,
                pool,
                resume_on_error,
            )?;
        }
    }
    Ok(())
}

// ============================================================================
// Statistics and Progress Tracking
// ============================================================================

/// Update data for statistics tracking
#[derive(Debug)]
pub struct StatisticsUpdate {
    uploaded: usize,
    new: usize,
    updated: usize,
    unchanged: usize,
    failed: usize,
}

/// Thread-safe statistics tracking for comment uploads
#[derive(Debug)]
pub struct Statistics {
    bytes_read: AtomicUsize,
    uploaded: AtomicUsize,
    new: AtomicUsize,
    updated: AtomicUsize,
    unchanged: AtomicUsize,
    annotations: AtomicUsize,
    failed_annotations: AtomicUsize,
    failed_comments: AtomicUsize,
    attachments: AtomicUsize,
    failed_attachments: AtomicUsize,
}

impl AnnotationStatistic for Statistics {
    fn add_annotation(&self) {
        self.annotations.fetch_add(1, Ordering::SeqCst);
    }
    fn add_failed_annotation(&self) {
        self.failed_annotations.fetch_add(1, Ordering::SeqCst);
    }
}
impl AttachmentStatistic for Statistics {
    fn add_attachment(&self) {
        self.attachments.fetch_add(1, Ordering::SeqCst);
    }
    fn add_failed_attachment(&self) {
        self.failed_attachments.fetch_add(1, Ordering::SeqCst);
    }
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
            failed_annotations: AtomicUsize::new(0),
            failed_comments: AtomicUsize::new(0),
            attachments: AtomicUsize::new(0),
            failed_attachments: AtomicUsize::new(0),
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
        self.failed_comments
            .fetch_add(update.failed, Ordering::SeqCst);
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

    #[inline]
    fn num_failed_annotations(&self) -> usize {
        self.failed_annotations.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_comments(&self) -> usize {
        self.failed_comments.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_attachments(&self) -> usize {
        self.attachments.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_attachments(&self) -> usize {
        self.failed_attachments.load(Ordering::SeqCst)
    }
}

/// Generate detailed progress statistics (only available when using --overwrite/sync endpoint)
fn detailed_statistics(statistics: &Statistics) -> (u64, String) {
    let bytes_read = statistics.bytes_read();
    let num_uploaded = statistics.num_uploaded();
    let num_new = statistics.num_new();
    let num_updated = statistics.num_updated();
    let num_unchanged = statistics.num_unchanged();
    let num_annotations = statistics.num_annotations();
    let num_failed_annotations = statistics.num_failed_annotations();
    let num_failed_comments = statistics.num_failed_comments();
    let num_attachments = statistics.num_attachments();
    let num_failed_attachments = statistics.num_failed_attachments();

    let failed_annotations_string = if num_failed_annotations > 0 {
        format!(" {num_failed_annotations} {}", "skipped".dimmed())
    } else {
        String::new()
    };
    let failed_attachments_string = if num_failed_attachments > 0 {
        format!(" {num_failed_attachments} {}", "skipped".dimmed())
    } else {
        String::new()
    };

    let attachments_string = if (num_attachments + num_failed_attachments) > 0 {
        format!(
            " [{num_attachments} {}{failed_attachments_string}]",
            "attachments".dimmed()
        )
    } else {
        String::new()
    };

    let failed_comments_string = if num_failed_comments > 0 {
        format!(" {num_failed_comments} {}", "skipped".dimmed())
    } else {
        String::new()
    };

    (
        bytes_read as u64,
        format!(
            "{} {}: {} {} {} {} {} {}{} [{} {}{}]{}",
            num_uploaded.to_string().bold(),
            "comments".dimmed(),
            num_new,
            "new".dimmed(),
            num_updated,
            "upd".dimmed(),
            num_unchanged,
            "nop".dimmed(),
            failed_comments_string,
            num_annotations,
            "annotations".dimmed(),
            failed_annotations_string,
            attachments_string
        ),
    )
}

/// Generate basic progress statistics (always available)
fn basic_statistics(statistics: &Statistics) -> (u64, String) {
    let bytes_read = statistics.bytes_read();
    let num_uploaded = statistics.num_uploaded();
    let num_annotations = statistics.num_annotations();
    let num_failed_annotations = statistics.num_failed_annotations();
    let num_failed_comments = statistics.num_failed_comments();
    let num_attachments = statistics.num_attachments();
    let num_failed_attachments = statistics.num_failed_attachments();

    let failed_annotations_string = if num_failed_annotations > 0 {
        format!(" {num_failed_annotations} {}", "skipped".dimmed())
    } else {
        String::new()
    };

    let failed_comments_string = if num_failed_comments > 0 {
        format!(" {num_failed_comments} {}", "skipped".dimmed())
    } else {
        String::new()
    };

    let failed_attachments_string = if num_failed_attachments > 0 {
        format!(" {num_failed_attachments} {}", "skipped".dimmed())
    } else {
        String::new()
    };

    let attachments_string = if (num_attachments + num_failed_attachments) > 0 {
        format!(
            " [{num_attachments} {}{failed_attachments_string}]",
            "attachments".dimmed()
        )
    } else {
        String::new()
    };

    (
        bytes_read as u64,
        format!(
            "{} {}{} [{} {}{}]{}",
            num_uploaded.to_string().bold(),
            "comments".dimmed(),
            failed_comments_string,
            num_annotations,
            "annotations".dimmed(),
            failed_annotations_string,
            attachments_string
        ),
    )
}

/// Create a progress bar with appropriate statistics formatter
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
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: true },
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::{check_no_duplicate_ids, read_comments_iter, Statistics};
    use std::io::{BufReader, Cursor};

    const SAMPLE_DUPLICATES: &str = include_str!("../../../tests/samples/duplicates.jsonl");

    #[test]
    fn test_read_comments_iter() {
        let reader = BufReader::new(Cursor::new(SAMPLE_DUPLICATES));
        let statistics = Statistics::new();

        let comments_iter = read_comments_iter(reader, Some(&statistics));

        assert_eq!(comments_iter.count(), 5);
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
