///! Task: transfer some labels from one dataset to another:
///!
///! For all comments in the target dataset with `only_for_target_label` assigned,
///! - copy all labels from the source dataset
///! - nest them under a new target dataset parent label: `transferred_label_parent`
///! - EXCEPT if the source label starts with the new parent label, don't nest it
///!
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use colored::Colorize;
use failchain::ResultExt;
use reinfer_client::{
    resources::comment::{Labelling, NewLabelling, Uid},
    AnnotatedComment, Client, Comment, DatasetIdentifier, Label, LabelName, Sentiment,
};
use structopt::StructOpt;

use crate::{
    errors::{ErrorKind, Result},
    progress::{Options as ProgressOptions, Progress},
};

#[derive(Debug, StructOpt)]
pub struct TakeMoreLabelsArgs {
    #[structopt(long = "source")]
    /// Dataset to take labels from.
    source_dataset: DatasetIdentifier,

    #[structopt(long = "target")]
    /// Dataset to merge labels into for reviewed comments.
    target_dataset: DatasetIdentifier,

    #[structopt(long = "only-for-target-label")]
    /// Only take more labels for target comments with this label.
    only_for_target_label: LabelName,

    #[structopt(long = "transferred-label-parent")]
    /// Prefix transferred labels with this parent label.
    transferred_label_parent: LabelName,
}

pub fn run(args: &TakeMoreLabelsArgs, client: &Client, experimental: bool) -> Result<()> {
    if !experimental {
        return Err(ErrorKind::Config(
            "Experimental features must be enabled for this task.".to_owned(),
        )
        .into());
    };
    let TakeMoreLabelsArgs {
        source_dataset,
        target_dataset,
        only_for_target_label,
        transferred_label_parent,
    } = args;
    let source_dataset = client
        .get_dataset(source_dataset.clone())
        .chain_err(|| ErrorKind::Client("Getting source dataset failed.".into()))?;
    let target_dataset = client
        .get_dataset(target_dataset.clone())
        .chain_err(|| ErrorKind::Client("Getting target dataset failed.".into()))?;

    let statistics = Arc::new(Statistics::default());
    let make_progress = || -> Result<Progress> { Ok(get_take_more_labels_progress(&statistics)) };
    make_progress()?;

    for source_id in target_dataset.source_ids.iter() {
        // Get target comments and their annotations
        for target_comment_batch in
            client.get_labellings_iter(&target_dataset.full_name(), source_id, false, None)
        {
            let target_comment_batch = target_comment_batch.chain_err(|| {
                ErrorKind::Client("Getting batch of target comments failed.".into())
            })?;
            statistics.add_target_comments(target_comment_batch.len());

            // If we care about the target comment, return a lookup of the comment id and existing
            // target labels
            let mut target_labellings_to_update =
                make_labellings_lookup(target_comment_batch, &only_for_target_label);

            // If we don't care about any comments in this batch, get next one
            if target_labellings_to_update.is_empty() {
                continue;
            }

            // Fetch source labellings for our given ids
            let source_annotated_comments = client
                .get_labellings(
                    &source_dataset.full_name(),
                    target_labellings_to_update.keys(),
                )
                .chain_err(|| {
                    ErrorKind::Client("Getting batch of source labellings failed.".into())
                })?;

            // For each set of source annotations we need to merge in
            for source_annotated_comment in source_annotated_comments.into_iter() {
                let AnnotatedComment {
                    comment: Comment { uid, .. },
                    labelling: source_labelling,
                    ..
                } = source_annotated_comment;
                let source_assigned = source_labelling.unwrap_or_default().assigned;
                if source_assigned.is_empty() {
                    continue;
                }
                // Get the matching dismissed annotations we already have
                let Labelling {
                    assigned: target_assigned,
                    dismissed,
                    ..
                } = target_labellings_to_update
                    .remove(&uid)
                    .expect("Tried to make a new labelling for a comment we don't need to update.");

                // Merge assigned labels from the two datasets
                let assigned = merge_assigned_labels(
                    target_assigned,
                    source_assigned,
                    &transferred_label_parent,
                );
                let new_labelling = NewLabelling {
                    assigned,
                    dismissed,
                };

                // Push the new merged annotations up to the API
                client
                    .update_labelling(
                        &target_dataset.full_name(),
                        &uid,
                        Some(&new_labelling),
                        None,
                    )
                    .chain_err(|| ErrorKind::Client("Updating labelling failed.".into()))?;
                statistics.add_labellings_updated(1);
            }

            let _progress = make_progress()?;
        }
    }
    Ok(())
}

/// Collect target comment ids and labellings, which contain the given assigend label.
fn make_labellings_lookup(
    annotated_comments: Vec<AnnotatedComment>,
    assigned_label_name: &LabelName,
) -> HashMap<Uid, Labelling> {
    let mut target_labellings_to_update = HashMap::new();
    // Consume the comments here as we break them up into parts for reuse
    for annotated_comment in annotated_comments.into_iter() {
        let AnnotatedComment {
            comment, labelling, ..
        } = annotated_comment;
        // If our target label has the label that indicates we need to take labels
        if let Some(labelling) = labelling {
            if labelling
                .assigned
                .iter()
                .any(|label| &label.name == assigned_label_name)
            {
                let Comment { uid, .. } = comment;
                target_labellings_to_update.insert(uid, labelling);
            }
        }
    }
    target_labellings_to_update
}

/// Merge the old and new assigned labels together.
///
/// Transferred labels will be nested under `transferred_label_parent` unless
/// they already begin with that parent label.
fn merge_assigned_labels(
    target_assigned: Vec<Label>,
    source_assigned: Vec<Label>,
    transferred_label_parent: &LabelName,
) -> Vec<Label> {
    let mut assigned = target_assigned;

    // We should push all new labels under this parent label
    // So add the parent, label always (we can dedupe later)
    assigned.push(Label {
        name: transferred_label_parent.clone(),
        // We just arbitrarily set Positive here, as we're operating on a sentimentless dataset
        sentiment: Sentiment::Positive,
    });

    // For faster checking, delimit our label name so we can use `starts_with`
    let label_prefix_check = format!("{} >", &transferred_label_parent.0);
    let renamed_source_labels = source_assigned.into_iter().map(|mut label| {
        if &label.name == transferred_label_parent || label.name.0.starts_with(&label_prefix_check)
        {
            // If our label begins with or is the new nesting parent, don't nest
            // (just merge in with the existing structure)
            label
        } else {
            // Otherwise, update the label name by nesting it
            label.name = LabelName(format!("{} > {}", transferred_label_parent.0, label.name.0));
            label
        }
    });
    assigned.extend(renamed_source_labels);

    // Deduplicate our assigned labels
    assigned.sort();
    assigned.dedup();
    assigned
}

#[derive(Debug, Default)]
struct Statistics {
    target_comments: AtomicUsize,
    labellings_updated: AtomicUsize,
}

impl Statistics {
    #[inline]
    pub fn add_target_comments(&self, n: usize) {
        self.target_comments.fetch_add(n, Ordering::SeqCst);
    }

    #[inline]
    pub fn add_labellings_updated(&self, n: usize) {
        self.labellings_updated.fetch_add(n, Ordering::SeqCst);
    }

    #[inline]
    pub fn target_comments(&self) -> usize {
        self.target_comments.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn labellings_updated(&self) -> usize {
        self.labellings_updated.load(Ordering::SeqCst)
    }
}

fn get_take_more_labels_progress(statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let target_comments = statistics.target_comments();
            (
                target_comments as u64,
                format!(
                    "{} {}, {} {}",
                    target_comments.to_string().bold(),
                    "checked".dimmed(),
                    statistics.labellings_updated().to_string().bold(),
                    "updated".dimmed(),
                ),
            )
        },
        &statistics,
        0,
        ProgressOptions {
            bytes_units: false,
            ..Default::default()
        },
    )
}
