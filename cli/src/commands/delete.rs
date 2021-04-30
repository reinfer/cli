use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use structopt::StructOpt;

use reinfer_client::{
    BucketIdentifier, Client, CommentId, CommentsIter, CommentsIterTimerange, DatasetIdentifier,
    Source, SourceIdentifier,
};

use crate::progress::{Options as ProgressOptions, Progress};

#[derive(Debug, StructOpt)]
pub enum DeleteArgs {
    #[structopt(name = "source")]
    /// Delete a source
    Source {
        #[structopt(name = "source")]
        /// Name or id of the source to delete
        source: SourceIdentifier,
    },

    #[structopt(name = "comments")]
    /// Delete comments by id in a source.
    Comments {
        #[structopt(short = "s", long = "source")]
        /// Name or id of the source to delete comments from
        source: SourceIdentifier,

        #[structopt(name = "comment id")]
        /// Ids of the comments to delete
        comments: Vec<CommentId>,
    },

    #[structopt(name = "bulk")]
    /// Delete all comments in a given time range.
    BulkComments {
        #[structopt(short = "s", long = "source")]
        /// Name or id of the source to delete comments from
        source: SourceIdentifier,

        #[structopt(long, parse(try_from_str))]
        /// Whether to delete comments that are annotated in any of the datasets
        /// containing this source.
        /// Use --include-annotated=false to keep any annotated comments in the given range.
        /// Use --include-annotated=true to delete all comments.
        include_annotated: bool,

        #[structopt(long)]
        /// Starting timestamp for comments to delete (inclusive). Should be in
        /// RFC 3339 format, e.g. 1970-01-02T03:04:05Z
        from_timestamp: Option<DateTime<Utc>>,

        #[structopt(long)]
        /// Ending timestamp for comments to delete (inclusive). Should be in
        /// RFC 3339 format, e.g. 1970-01-02T03:04:05Z
        to_timestamp: Option<DateTime<Utc>>,

        #[structopt(long)]
        /// Don't display a progress bar
        no_progress: bool,
    },

    #[structopt(name = "bucket")]
    /// Delete a bucket
    Bucket {
        #[structopt(name = "bucket")]
        /// Name or id of the bucket to delete
        bucket: BucketIdentifier,
    },

    #[structopt(name = "dataset")]
    /// Delete a dataset
    Dataset {
        #[structopt(name = "dataset")]
        /// Name or id of the dataset to delete
        dataset: DatasetIdentifier,
    },
}

pub fn run(delete_args: &DeleteArgs, client: Client) -> Result<()> {
    match delete_args {
        DeleteArgs::Source { source } => {
            client
                .delete_source(source.clone())
                .context("Operation to delete source has failed.")?;
            log::info!("Deleted source.");
        }
        DeleteArgs::Comments { source, comments } => {
            client
                .delete_comments(source.clone(), comments)
                .context("Operation to delete comments has failed.")?;
            log::info!("Deleted comments.");
        }
        DeleteArgs::BulkComments {
            source: source_identifier,
            include_annotated,
            from_timestamp,
            to_timestamp,
            no_progress,
        } => {
            let source = client.get_source(source_identifier.clone())?;
            let show_progress = !no_progress;
            delete_comments_in_period(
                &client,
                source,
                *include_annotated,
                CommentsIterTimerange {
                    from: *from_timestamp,
                    to: *to_timestamp,
                },
                show_progress,
            )
            .context("Operation to delete comments has failed.")?;
        }
        DeleteArgs::Dataset { dataset } => {
            client
                .delete_dataset(dataset.clone())
                .context("Operation to delete dataset has failed.")?;
            log::info!("Deleted dataset.");
        }
        DeleteArgs::Bucket { bucket } => {
            client
                .delete_bucket(bucket.clone())
                .context("Operation to delete bucket has failed.")?;
            log::info!("Deleted bucket.");
        }
    };
    Ok(())
}

fn delete_comments_in_period(
    client: &Client,
    source: Source,
    include_annotated: bool,
    timerange: CommentsIterTimerange,
    show_progress: bool,
) -> Result<()> {
    log::info!(
        "Deleting comments in source `{}`{} (include-annotated: {})",
        source.full_name().0,
        match (timerange.from, timerange.to) {
            (None, None) => "".into(),
            (Some(start), None) => format!(" after {}", start),
            (None, Some(end)) => format!(" before {}", end),
            (Some(start), Some(end)) => format!(" in range {} -> {}", start, end),
        },
        include_annotated,
    );
    let statistics = Arc::new(Statistics::new());
    {
        // Deleting comments in a block to ensure `_progress` is dropped before
        // logging any further
        let _progress = if show_progress {
            Some(delete_comments_progress_bar(&statistics))
        } else {
            None
        };

        // This is the maximum number of comments which the API permits deleting in a single call.
        const DELETION_BATCH_SIZE: usize = 128;
        // Buffer to store comment IDs to delete - allow it to be slightly larger than the deletion
        // batch size so that if there's an incomplete page it'll increase the counts.
        let mut comments_to_delete =
            Vec::with_capacity(DELETION_BATCH_SIZE + CommentsIter::MAX_PAGE_SIZE);

        let delete_batch = |comment_ids: Vec<CommentId>| -> Result<()> {
            client
                .delete_comments(&source, &comment_ids)
                .context("Operation to delete comments failed")?;
            statistics.increment_deleted(comment_ids.len());
            Ok(())
        };

        client
            .get_comments_iter(
                &source.full_name(),
                Some(CommentsIter::MAX_PAGE_SIZE),
                timerange,
            )
            .try_for_each(|page| -> Result<()> {
                let page = page.context("Operation to get comments failed")?;
                let num_comments = page.len();
                let comment_ids = page
                    .into_iter()
                    .filter_map(|comment| {
                        if !include_annotated && comment.has_annotations {
                            None
                        } else {
                            Some(comment.id)
                        }
                    })
                    .collect::<Vec<_>>();

                let num_skipped = num_comments - comment_ids.len();
                statistics.increment_skipped(num_skipped);

                comments_to_delete.extend(comment_ids);
                while comments_to_delete.len() >= DELETION_BATCH_SIZE {
                    let remainder = comments_to_delete.split_off(DELETION_BATCH_SIZE);
                    delete_batch(std::mem::replace(&mut comments_to_delete, remainder))?;
                }
                Ok(())
            })?;

        // Delete any comments left over in any potential last partial batch.
        if !comments_to_delete.is_empty() {
            assert!(comments_to_delete.len() < DELETION_BATCH_SIZE);
            delete_batch(comments_to_delete)?;
        }
    }
    log::info!(
        "Deleted {} comments (skipped {}).",
        statistics.deleted(),
        statistics.skipped()
    );
    Ok(())
}

#[derive(Debug)]
pub struct Statistics {
    deleted: AtomicUsize,
    skipped: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            deleted: AtomicUsize::new(0),
            skipped: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn increment_deleted(&self, num_deleted: usize) {
        self.deleted.fetch_add(num_deleted, Ordering::SeqCst);
    }

    #[inline]
    fn increment_skipped(&self, num_skipped: usize) {
        self.skipped.fetch_add(num_skipped, Ordering::SeqCst);
    }

    #[inline]
    fn deleted(&self) -> usize {
        self.deleted.load(Ordering::SeqCst)
    }

    #[inline]
    fn skipped(&self) -> usize {
        self.skipped.load(Ordering::SeqCst)
    }
}

fn delete_comments_progress_bar(statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let num_deleted = statistics.deleted() as u64;
            let num_skipped = statistics.skipped() as u64;
            (
                num_deleted + num_skipped,
                format!(
                    "{} {}{}",
                    num_deleted.to_string().bold(),
                    "deleted".dimmed(),
                    format!(" [{} {}] total", num_skipped, "skipped".dimmed())
                ),
            )
        },
        &statistics,
        None,
        ProgressOptions { bytes_units: false },
    )
}
