use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use reinfer_client::{
    AnnotatedComment, Client, CommentId, CommentsIterTimerange, DatasetFullName, DatasetIdentifier,
    HasAnnotations, Source, SourceIdentifier,
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

use crate::{
    printer::print_resources_as_json,
    progress::{Options as ProgressOptions, Progress},
};

#[derive(Debug, StructOpt)]
pub struct GetSingleCommentArgs {
    #[structopt(long = "source")]
    /// Source name or id
    source: SourceIdentifier,

    #[structopt(name = "comment-id")]
    /// Comment id.
    comment_id: CommentId,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write comments as JSON. If not specified, stdout will be used.
    path: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct GetManyCommentsArgs {
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
}

pub fn get_single(client: &Client, args: &GetSingleCommentArgs) -> Result<()> {
    let GetSingleCommentArgs {
        source,
        comment_id,
        path,
    } = args;
    let file: Option<Box<dyn Write>> = match path {
        Some(path) => Some(Box::new(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
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
            moon_forms: None,
        }),
        &mut writer,
    )
}

pub fn get_many(client: &Client, args: &GetManyCommentsArgs) -> Result<()> {
    let GetManyCommentsArgs {
        source,
        dataset,
        no_progress,
        include_predictions,
        reviewed_only,
        from_timestamp,
        to_timestamp,
        path,
    } = args;

    let by_timerange = from_timestamp.is_some() || to_timestamp.is_some();
    if reviewed_only.unwrap_or_default() && by_timerange {
        bail!("The `reviewed_only` and `from/to-timestamp` options are mutually exclusive.")
    }

    if reviewed_only.unwrap_or_default() && dataset.is_none() {
        bail!("Cannot get reviewed comments when `dataset` is not provided.")
    }

    if include_predictions.unwrap_or_default() && dataset.is_none() {
        bail!("Cannot get predictions when `dataset` is not provided.")
    }

    let file = match path {
        Some(path) => Some(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
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
        download_comments(client, source.clone(), file, download_options)
    } else {
        download_comments(
            client,
            source.clone(),
            io::stdout().lock(),
            download_options,
        )
    }
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
                *client
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
                options.timerange,
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
                        moon_forms: None,
                    }),
                    &mut writer,
                )
            })?;
    }
    log::info!(
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
    timerange: CommentsIterTimerange,
) -> Result<()> {
    client
        .get_comments_iter(&source.full_name(), None, timerange)
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
