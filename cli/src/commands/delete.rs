use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use log::info;
use structopt::StructOpt;

use openapi::{
    apis::{
        buckets_api::{delete_bucket, delete_keyed_sync_state, query_keyed_sync_state_ids},
        comments_api::delete_comment,
        configuration::Configuration,
        datasets_api::{delete_dataset_by_id, delete_dataset_v1},
        projects_api::delete_project,
        sources_api::delete_source,
        users_api::delete_user,
    },
    models::{QueryKeyedSyncStateIdsRequest, Source},
};

use crate::commands::get::comments::{get_comments_iter, CommentsIter};
use crate::progress::{Options as ProgressOptions, Progress};
use crate::utils::{
    types::identifiers::{resolve_bucket, resolve_source, ResourceIdentifier, UserIdentifier},
    BucketIdentifier, CommentId, CommentsIterTimerange, DatasetIdentifier, ProjectName,
    SourceIdentifier,
};

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

    #[structopt(name = "user")]
    /// Delete a user
    User {
        #[structopt(name = "user")]
        /// Username or id of the user to delete
        user: UserIdentifier,
    },

    #[structopt(name = "project")]
    /// Delete a project
    Project {
        #[structopt(name = "project")]
        /// Name or id of the project to delete
        project: ProjectName,

        #[structopt(long)]
        /// Force deletion of the project, even if it's not empty.
        force: bool,
    },

    #[structopt(name = "keyed-sync-states")]
    /// Delete keyed sync states
    KeyedSyncStates {
        /// The bucket to delete keyed sync states for
        bucket: BucketIdentifier,

        /// The mailbox to delete keyed sync states for
        mailbox_name: String,
    },
}

pub fn run(delete_args: &DeleteArgs, config: &Configuration) -> Result<()> {
    match delete_args {
        DeleteArgs::Source { source } => {
            // Resolve source identifier to get the ID
            let resolved_source =
                resolve_source(config, source).context("Failed to resolve source")?;

            delete_source(config, &resolved_source.id)
                .context("Operation to delete source has failed.")?;
            log::info!("Deleted source.");
        }
        DeleteArgs::User { user } => {
            let user_id = user
                .id()
                .ok_or_else(|| anyhow::anyhow!("User must be specified by ID"))?;
            delete_user(config, user_id).context("Operation to delete user has failed.")?;
            log::info!("Deleted user.");
        }
        DeleteArgs::Comments { source, comments } => {
            // Delete comments one by one
            for comment_id in comments {
                let owner = source
                    .owner()
                    .ok_or_else(|| anyhow::anyhow!("Source must be specified by full name"))?;
                let name = source
                    .name()
                    .ok_or_else(|| anyhow::anyhow!("Source must be specified by full name"))?;
                delete_comment(config, owner, name, None, Some(&comment_id.0)).context(format!(
                    "Operation to delete comment {} has failed.",
                    comment_id
                ))?;
            }
            log::info!("Deleted comments.");
        }
        DeleteArgs::BulkComments {
            source: source_identifier,
            include_annotated,
            from_timestamp,
            to_timestamp,
            no_progress,
        } => {
            let source =
                resolve_source(config, source_identifier).context("Failed to resolve source")?;
            let show_progress = !no_progress;
            delete_comments_in_period(
                config,
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
            match dataset {
                ResourceIdentifier::Id(dataset_id) => {
                    delete_dataset_by_id(config, dataset_id)
                        .context("Operation to delete dataset has failed.")?;
                }
                ResourceIdentifier::FullName(full_name) => {
                    delete_dataset_v1(config, &full_name.owner(), &full_name.name())
                        .context("Operation to delete dataset has failed.")?;
                }
            }
            log::info!("Deleted dataset.");
        }
        DeleteArgs::Bucket { bucket } => {
            let bucket_id = bucket
                .id()
                .ok_or_else(|| anyhow::anyhow!("Bucket must be specified by ID"))?;
            delete_bucket(config, bucket_id).context("Operation to delete bucket has failed.")?;
            log::info!("Deleted bucket.");
        }
        DeleteArgs::Project { project, force } => {
            delete_project(config, project.as_str(), Some(*force)).map_err(|e| {
                match e {
                    openapi::apis::Error::ResponseError(response_content) => {
                        // Try to parse the error response to get the detailed message
                        let detailed_error = if let Ok(error_resp) =
                            serde_json::from_str::<openapi::models::ErrorResponse>(
                                &response_content.content,
                            ) {
                            error_resp.message
                        } else {
                            response_content.content
                        };
                        anyhow::anyhow!(
                            "Operation to delete project has failed: {}",
                            detailed_error
                        )
                    }
                    _ => anyhow::anyhow!("Operation to delete project has failed: {}", e),
                }
            })?;
            log::info!("Deleted project.");
        }
        DeleteArgs::KeyedSyncStates {
            bucket,
            mailbox_name,
        } => {
            let bucket = resolve_bucket(config, bucket).context("Failed to resolve bucket")?;

            let keyed_sync_state_ids_response = query_keyed_sync_state_ids(
                config,
                &bucket.id,
                QueryKeyedSyncStateIdsRequest {
                    mailbox_name: mailbox_name.clone(),
                },
            )
            .context("Failed to get keyed sync state IDs")?;

            for id in keyed_sync_state_ids_response.keyed_sync_state_ids {
                delete_keyed_sync_state(config, &bucket.id, &id)
                    .context("Failed to delete keyed sync state")?;
                info!("Delete keyed sync state {}", id)
            }
        }
    };
    Ok(())
}

fn delete_comments_in_period(
    config: &Configuration,
    source: Source,
    include_annotated: bool,
    timerange: CommentsIterTimerange,
    show_progress: bool,
) -> Result<()> {
    log::info!(
        "Deleting comments in source `{}/{}`{} (include-annotated: {})",
        source.owner,
        source.name,
        match (timerange.from, timerange.to) {
            (None, None) => "".into(),
            (Some(start), None) => format!(" after {start}"),
            (None, Some(end)) => format!(" before {end}"),
            (Some(start), Some(end)) => format!(" in range {start} -> {end}"),
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
        const DELETION_BATCH_SIZE: usize = 32;
        // Buffer to store comment IDs to delete - allow it to be slightly larger than the deletion
        // batch size so that if there's an incomplete page it'll increase the counts.
        let mut comments_to_delete =
            Vec::with_capacity(DELETION_BATCH_SIZE + CommentsIter::MAX_PAGE_SIZE);

        let delete_batch = |comment_ids: Vec<CommentId>| -> Result<()> {
            // Delete comments individually using OpenAPI
            for comment_id in &comment_ids {
                delete_comment(
                    config,
                    &source.owner,
                    &source.name,
                    None,
                    Some(&comment_id.0),
                )
                .context(format!("Operation to delete comment {} failed", comment_id))?;
            }
            statistics.increment_deleted(comment_ids.len());
            Ok(())
        };

        get_comments_iter(
            config,
            source.owner.clone(),
            source.name.clone(),
            Some(CommentsIter::MAX_PAGE_SIZE),
            timerange,
        )
        .try_for_each(|page| -> Result<()> {
            let page = page.context("Operation to get comments failed")?;
            let num_comments = page.len();
            let comment_ids = page
                .into_iter()
                .filter_map(|comment| {
                    if !include_annotated && comment.has_annotations.unwrap_or(false) {
                        None
                    } else {
                        Some(CommentId(comment.id))
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
                    "{} {} [{} {}] total",
                    num_deleted.to_string().bold(),
                    "deleted".dimmed(),
                    num_skipped,
                    "skipped".dimmed()
                ),
            )
        },
        statistics,
        None,
        ProgressOptions { bytes_units: false },
    )
}
