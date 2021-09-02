use crate::{
    printer::{print_resources_as_json, Printer},
    progress::{Options as ProgressOptions, Progress},
};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use log::info;
use reinfer_client::{
    AnnotatedComment, BucketIdentifier, Client, CommentId, CommentsIterTimerange, DatasetFullName,
    DatasetIdentifier, Source, SourceIdentifier, TriggerFullName,
};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum GetArgs {
    #[structopt(name = "datasets")]
    /// List available datasets
    Datasets {
        #[structopt(name = "dataset")]
        /// If specified, only list this dataset (name or id)
        dataset: Option<DatasetIdentifier>,
    },

    #[structopt(name = "sources")]
    /// List available sources
    Sources {
        #[structopt(name = "source")]
        /// If specified, only list this source (name or id)
        source: Option<SourceIdentifier>,
    },

    #[structopt(name = "comment")]
    /// Get a comments in a source
    Comment {
        #[structopt(long = "source")]
        /// Source name or id
        source: SourceIdentifier,

        #[structopt(name = "comment-id")]
        /// Comment id.
        comment_id: CommentId,

        #[structopt(short = "f", long = "file", parse(from_os_str))]
        /// Path where to write comments as JSON. If not specified, stdout will be used.
        path: Option<PathBuf>,
    },

    #[structopt(name = "comments")]
    /// Download all comments in a source
    Comments {
        #[structopt(name = "source")]
        /// Source name or id
        source: SourceIdentifier,

        #[structopt(short = "d", long = "dataset")]
        /// Dataset name or id
        dataset: Option<DatasetIdentifier>,

        #[structopt(long)]
        /// Don't display a progress bar (only applicable when --file is used).
        no_progress: bool,

        #[structopt(long = "predictions")]
        /// Save predicted labels and entities for each comment.
        include_predictions: Option<bool>,

        #[structopt(long = "reviewed-only")]
        /// Download reviewed comments only.
        reviewed_only: Option<bool>,

        #[structopt(long = "from-timestamp")]
        /// Starting timestamp for comments to retrieve (inclusive).
        from_timestamp: Option<DateTime<Utc>>,

        #[structopt(long = "to-timestamp")]
        /// Ending timestamp for comments to retrieve (inclusive).
        to_timestamp: Option<DateTime<Utc>>,

        #[structopt(short = "f", long = "file", parse(from_os_str))]
        /// Path where to write comments as JSON. If not specified, stdout will be used.
        path: Option<PathBuf>,
    },

    #[structopt(name = "buckets")]
    /// List available buckets
    Buckets {
        #[structopt(name = "bucket")]
        /// If specified, only list this bucket (name or id)
        bucket: Option<BucketIdentifier>,
    },

    #[structopt(name = "triggers")]
    /// List available triggers for a dataset
    Triggers {
        #[structopt(short = "d", long = "dataset")]
        /// The dataset name or id
        dataset: DatasetIdentifier,
    },

    #[structopt(name = "trigger-comments")]
    /// Fetch comments from a trigger
    TriggerComments {
        #[structopt(short = "t", long = "trigger")]
        /// The full trigger name `<owner>/<dataset>/<trigger>`.
        trigger: TriggerFullName,

        #[structopt(short = "s", long = "size", default_value = "16")]
        /// The max number of comments to return per batch.
        size: u32,

        #[structopt(short = "l", long = "listen")]
        /// If set, the command will run forever polling every N seconds and advancing the trigger.
        listen: Option<f64>,

        #[structopt(long = "individual-advance")]
        /// If set, the command will acknowledge each comment in turn, rather than full batches.
        individual_advance: bool,
    },

    #[structopt(name = "users")]
    /// List available users
    Users,

    #[structopt(name = "current-user")]
    /// Get the user associated with the API token in use
    CurrentUser,
}

pub fn run(get_args: &GetArgs, client: Client, printer: &Printer) -> Result<()> {
    match get_args {
        GetArgs::Datasets { dataset } => {
            let datasets = if let Some(dataset) = dataset {
                vec![client
                    .get_dataset(dataset.clone())
                    .context("Operation to list datasets has failed.")?]
            } else {
                let mut datasets = client
                    .get_datasets()
                    .context("Operation to list datasets has failed.")?;
                datasets.sort_unstable_by(|lhs, rhs| {
                    (&lhs.owner.0, &lhs.name.0).cmp(&(&rhs.owner.0, &rhs.name.0))
                });
                datasets
            };
            printer.print_resources(&datasets)?;
        }
        GetArgs::Sources { source } => {
            let sources = if let Some(source) = source {
                vec![client
                    .get_source(source.clone())
                    .context("Operation to list sources has failed.")?]
            } else {
                let mut sources = client
                    .get_sources()
                    .context("Operation to list sources has failed.")?;
                sources.sort_unstable_by(|lhs, rhs| {
                    (&lhs.owner.0, &lhs.name.0).cmp(&(&rhs.owner.0, &rhs.name.0))
                });
                sources
            };
            printer.print_resources(&sources)?;
        }
        GetArgs::Comment {
            source,
            comment_id,
            path,
        } => {
            let file: Option<Box<dyn Write>> = match path {
                Some(path) => Some(Box::new(
                    File::create(path)
                        .with_context(|| {
                            format!("Could not open file for writing `{}`", path.display())
                        })
                        .map(BufWriter::new)?,
                )),
                None => None,
            };

            let stdout = io::stdout();
            let mut writer: Box<dyn Write> = file.unwrap_or_else(|| Box::new(stdout.lock()));
            let source = client
                .get_source(source.to_owned())
                .context("Operation to get source has failed.")?;
            let comment = client.get_comment(&source.full_name(), comment_id)?;
            print_resources_as_json(
                std::iter::once(AnnotatedComment {
                    comment,
                    labelling: None,
                    entities: None,
                    thread_properties: None,
                }),
                &mut writer,
            )?
        }
        GetArgs::Comments {
            source,
            dataset,
            no_progress,
            include_predictions,
            reviewed_only,
            from_timestamp,
            to_timestamp,
            path,
        } => {
            let by_timerange = from_timestamp.is_some() || to_timestamp.is_some();
            if dataset.is_some() && by_timerange {
                return Err(anyhow!(
                    "The `dataset` and `from/to-timestamp` options are mutually exclusive."
                ));
            }
            let file = match path {
                Some(path) => Some(
                    File::create(path)
                        .with_context(|| {
                            format!("Could not open file for writing `{}`", path.display())
                        })
                        .map(BufWriter::new)?,
                ),
                None => None,
            };

            let download_options = CommentDownloadOptions {
                dataset_identifier: dataset.clone(),
                include_predictions: include_predictions.unwrap_or(false),
                reviewed_only: reviewed_only.unwrap_or(false),
                timerange: CommentsIterTimerange {
                    from: *from_timestamp,
                    to: *to_timestamp,
                },
                show_progress: !no_progress,
            };

            if let Some(file) = file {
                download_comments(&client, source.clone(), file, download_options)?;
            } else {
                download_comments(
                    &client,
                    source.clone(),
                    io::stdout().lock(),
                    download_options,
                )?;
            }
        }
        GetArgs::Buckets { bucket } => {
            let buckets = if let Some(bucket) = bucket {
                vec![client
                    .get_bucket(bucket.clone())
                    .context("Operation to list buckets has failed.")?]
            } else {
                let mut buckets = client
                    .get_buckets()
                    .context("Operation to list buckets has failed.")?;
                buckets.sort_unstable_by(|lhs, rhs| {
                    (&lhs.owner.0, &lhs.name.0).cmp(&(&rhs.owner.0, &rhs.name.0))
                });
                buckets
            };
            printer.print_resources(&buckets)?;
        }
        GetArgs::Triggers { dataset } => {
            let dataset_name = client
                .get_dataset(dataset.clone())
                .context("Operation to get dataset has failed.")?
                .full_name();
            let mut triggers = client
                .get_triggers(&dataset_name)
                .context("Operation to list triggers has failed.")?;
            triggers.sort_unstable_by(|lhs, rhs| lhs.name.0.cmp(&rhs.name.0));
            printer.print_resources(&triggers)?;
        }
        GetArgs::TriggerComments {
            trigger,
            size,
            listen,
            individual_advance,
        } => match listen {
            Some(delay) => loop {
                let batch = client
                    .fetch_trigger_comments(trigger, *size)
                    .context("Operation to fetch trigger comments failed.")?;
                if batch.results.is_empty() {
                    if batch.filtered == 0 {
                        std::thread::sleep(std::time::Duration::from_secs_f64(*delay));
                    } else {
                        client
                            .advance_trigger(trigger, batch.sequence_id)
                            .context("Operation to advance trigger for batch failed.")?;
                    }
                    continue;
                }
                let needs_final_advance = !individual_advance
                    || batch.sequence_id != batch.results.last().unwrap().sequence_id;
                for result in batch.results {
                    print_resources_as_json(Some(&result), io::stdout().lock())?;

                    if *individual_advance {
                        client
                            .advance_trigger(trigger, result.sequence_id)
                            .context("Operation to advance trigger for comment failed.")?;
                    }
                }
                if needs_final_advance {
                    client
                        .advance_trigger(trigger, batch.sequence_id)
                        .context("Operation to advance trigger for batch failed.")?;
                }
            },
            None => {
                let batch = client
                    .fetch_trigger_comments(trigger, *size)
                    .context("Operation to fetch trigger comments failed.")?;
                print_resources_as_json(Some(&batch), io::stdout().lock())?;
            }
        },
        GetArgs::Users {} => {
            let users = client
                .get_users()
                .context("Operation to list users has failed.")?;
            printer.print_resources(&users)?;
        }
        GetArgs::CurrentUser {} => {
            let user = client
                .get_current_user()
                .context("Operation to get the current user has failed.")?;
            printer.print_resources(&[user])?;
        }
    }
    Ok(())
}

#[derive(Default)]
struct CommentDownloadOptions {
    dataset_identifier: Option<DatasetIdentifier>,
    include_predictions: bool,
    reviewed_only: bool,
    timerange: CommentsIterTimerange,
    show_progress: bool,
}

fn download_comments(
    client: &Client,
    source_identifier: SourceIdentifier,
    mut writer: impl Write,
    options: CommentDownloadOptions,
) -> Result<()> {
    let source = client
        .get_source(source_identifier)
        .context("Operation to get source has failed.")?;
    let statistics = Arc::new(Statistics::new());
    let make_progress = |dataset_name: Option<&DatasetFullName>| -> Result<Progress> {
        Ok(get_comments_progress_bar(
            if let Some(dataset_name) = dataset_name {
                client
                    .get_statistics(dataset_name)
                    .context("Operation to get comment count has failed..")?
                    .num_comments as u64
            } else {
                0
            },
            &statistics,
            dataset_name.is_some(),
        ))
    };

    if let Some(dataset_identifier) = options.dataset_identifier {
        let dataset = client
            .get_dataset(dataset_identifier)
            .context("Operation to get dataset has failed.")?;
        let dataset_name = dataset.full_name();
        let _progress = if options.show_progress {
            Some(make_progress(Some(&dataset_name))?)
        } else {
            None
        };

        if options.reviewed_only {
            get_reviewed_comments_in_bulk(
                client,
                dataset_name,
                source,
                &statistics,
                options.include_predictions,
                writer,
            )?;
        } else {
            get_comments_from_uids(
                client,
                dataset_name,
                source,
                &statistics,
                options.include_predictions,
                writer,
            )?;
        }
    } else {
        let _progress = if options.show_progress {
            Some(make_progress(None)?)
        } else {
            None
        };
        client
            .get_comments_iter(&source.full_name(), None, options.timerange)
            .try_for_each(|page| {
                let page = page.context("Operation to get comments has failed.")?;
                statistics.add_comments(page.len());
                print_resources_as_json(
                    page.into_iter().map(|comment| AnnotatedComment {
                        comment,
                        labelling: None,
                        entities: None,
                        thread_properties: None,
                    }),
                    &mut writer,
                )
            })?;
    }
    info!(
        "Successfully downloaded {} comments [{} annotated].",
        statistics.num_downloaded(),
        statistics.num_annotated(),
    );
    Ok(())
}

fn get_comments_from_uids(
    client: &Client,
    dataset_name: DatasetFullName,
    source: Source,
    statistics: &Arc<Statistics>,
    include_predictions: bool,
    mut writer: impl Write,
) -> Result<()> {
    client
        .get_comments_iter(&source.full_name(), None, Default::default())
        .try_for_each(|page| {
            let page = page.context("Operation to get comments has failed.")?;
            if page.is_empty() {
                return Ok(());
            }

            statistics.add_comments(page.len());
            let annotations =
                client.get_labellings(&dataset_name, page.iter().map(|comment| &comment.uid));
            let comments = page
                .into_iter()
                .zip(
                    annotations
                        .context("Operation to get comments has failed.")?
                        .into_iter(),
                )
                .map(|(comment, mut annotated)| {
                    if !include_predictions {
                        annotated = annotated.without_predictions();
                    }
                    annotated.comment = comment;
                    if annotated.has_annotations() {
                        statistics.add_annotated(1);
                    }
                    annotated
                });
            print_resources_as_json(comments, &mut writer)
        })?;
    Ok(())
}

fn get_reviewed_comments_in_bulk(
    client: &Client,
    dataset_name: DatasetFullName,
    source: Source,
    statistics: &Arc<Statistics>,
    include_predictions: bool,
    mut writer: impl Write,
) -> Result<()> {
    client
        .get_labellings_iter(&dataset_name, &source.id, include_predictions, None)
        .try_for_each(|page| {
            let page = page.context("Operation to get labellings has failed.")?;
            statistics.add_comments(page.len());
            statistics.add_annotated(page.len());
            let comments = page.into_iter().map(|comment| {
                if !include_predictions {
                    comment.without_predictions()
                } else {
                    comment
                }
            });
            print_resources_as_json(comments, &mut writer)
        })?;
    Ok(())
}

#[derive(Debug)]
pub struct Statistics {
    downloaded: AtomicUsize,
    annotated: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            downloaded: AtomicUsize::new(0),
            annotated: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_comments(&self, num_downloaded: usize) {
        self.downloaded.fetch_add(num_downloaded, Ordering::SeqCst);
    }

    #[inline]
    fn add_annotated(&self, num_downloaded: usize) {
        self.annotated.fetch_add(num_downloaded, Ordering::SeqCst);
    }

    #[inline]
    fn num_downloaded(&self) -> usize {
        self.downloaded.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_annotated(&self) -> usize {
        self.annotated.load(Ordering::SeqCst)
    }
}

fn get_comments_progress_bar(
    total_bytes: u64,
    statistics: &Arc<Statistics>,
    show_annotated: bool,
) -> Progress {
    Progress::new(
        move |statistics| {
            let num_downloaded = statistics.num_downloaded();
            let num_annotated = statistics.num_annotated();
            (
                num_downloaded as u64,
                format!(
                    "{} {}{}",
                    num_downloaded.to_string().bold(),
                    "comments".dimmed(),
                    if show_annotated {
                        format!(" [{} {}]", num_annotated, "annotated".dimmed())
                    } else {
                        "".into()
                    }
                ),
            )
        },
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: false },
    )
}
