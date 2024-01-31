use anyhow::{anyhow, bail, Context, Error, Result};

use chrono::{DateTime, Utc};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use log::info;
use regex::Regex;
use reinfer_client::{
    resources::{
        comment::{
            CommentTimestampFilter, MessagesFilter, PredictedLabelName, PropertyFilterKind,
            ReviewedFilterEnum, UserPropertiesFilter,
        },
        dataset::{
            Attribute, AttributeFilter, AttributeFilterEnum, OrderEnum, QueryRequestParams,
            StatisticsRequestParams as DatasetStatisticsRequestParams, SummaryRequestParams,
        },
        source::StatisticsRequestParams as SourceStatisticsRequestParams,
    },
    AnnotatedComment, Client, CommentFilter, CommentId, CommentsIterTimerange, DatasetFullName,
    DatasetIdentifier, Entities, HasAnnotations, LabelName, Labelling, ModelVersion,
    PredictedLabel, PropertyValue, Source, SourceIdentifier, DEFAULT_LABEL_GROUP_NAME,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    str::FromStr,
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

    #[structopt(long = "model-version")]
    /// Get predicted labels and entities from the specified model version rather than latest.
    model_version: Option<u32>,

    #[structopt(long = "reviewed-only")]
    /// Download reviewed comments only.
    reviewed_only: Option<bool>,

    #[structopt(long = "from-timestamp")]
    /// Starting timestamp for comments to retrieve (inclusive).
    from_timestamp: Option<DateTime<Utc>>,

    #[structopt(long = "to-timestamp")]
    /// Ending timestamp for comments to retrieve (inclusive).
    to_timestamp: Option<DateTime<Utc>>,

    #[structopt(long = "senders")]
    /// Filter to comments only from these senders
    senders: Option<Vec<String>>,

    #[structopt(long = "recipients")]
    /// Filter to emails only to these recipients
    recipients: Option<Vec<String>>,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write comments as JSON. If not specified, stdout will be used.
    path: Option<PathBuf>,

    #[structopt(short = "l", long = "label-filter")]
    /// Regex filter to select which labels you want to download predictions for
    label_filter: Option<Regex>,

    #[structopt(short = "p", long = "user-property-filter")]
    /// The user property filter to use as a json string
    property_filter: Option<StructExt<UserPropertiesFilter>>,

    #[structopt(long = "interactive-user-property-filter")]
    /// Open a dialog to interactively construct the user property filter to use
    interactive_property_filter: bool,
}

#[derive(Debug, Deserialize)]
struct StructExt<T>(pub T);

impl<T: serde::de::DeserializeOwned> FromStr for StructExt<T> {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        serde_json::from_str(string).map_err(|source| {
            anyhow!(
                "Expected valid json for type. Got: '{}', which failed because: '{}'",
                string.to_owned(),
                source
            )
        })
    }
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
            label_properties: None,
        }),
        &mut writer,
    )
}

fn get_user_properties_filter_interactively(
    client: &Client,
    dataset_id: DatasetIdentifier,
) -> Result<UserPropertiesFilter> {
    let dataset = client.get_dataset(dataset_id)?;

    let dataset_summary = client.dataset_summary(
        &dataset.full_name(),
        &SummaryRequestParams {
            attribute_filters: Vec::new(),
            filter: CommentFilter::default(),
        },
    )?;

    let mut properties: Vec<String> = dataset_summary
        .summary
        .user_properties
        .string
        .iter()
        .map(|p| p.full_name.clone())
        .collect();

    let mut filters = HashMap::new();

    loop {
        if properties.is_empty() {
            info!("No user properties available!");
            break;
        }

        let selected_property = Select::new()
            .with_prompt("Which user property would you like to filter?")
            .items(&properties)
            .interact()?;

        let selected_property_name = properties.remove(selected_property);

        let mut values: Vec<PropertyValue> = Vec::new();
        loop {
            let value: String = Input::new()
                .with_prompt("What value would you like to filter on?")
                .interact()?;
            values.push(PropertyValue::String(value));

            if !Confirm::new()
                .with_prompt(format!(
                    "Do you want to filter '{selected_property_name}' on more values?",
                ))
                .interact()?
            {
                break;
            }
        }

        filters.insert(selected_property_name, PropertyFilterKind::OneOf(values));

        if !Confirm::new()
            .with_prompt("Do you want to filter additional user properties?")
            .interact()?
        {
            break;
        }
    }

    Ok(UserPropertiesFilter(filters))
}

pub fn get_many(client: &Client, args: &GetManyCommentsArgs) -> Result<()> {
    let GetManyCommentsArgs {
        source,
        dataset,
        no_progress,
        include_predictions,
        model_version,
        reviewed_only,
        from_timestamp,
        to_timestamp,
        path,
        label_filter,
        property_filter: user_property_filter,
        interactive_property_filter: interative_property_filter,
        recipients,
        senders,
    } = args;

    let by_timerange = from_timestamp.is_some() || to_timestamp.is_some();
    if reviewed_only.unwrap_or_default() && by_timerange {
        bail!("The `reviewed_only` and `from/to-timestamp` options are mutually exclusive.")
    }

    let reviewed_only = reviewed_only.unwrap_or(false);
    if reviewed_only && model_version.is_some() {
        bail!("The `reviewed_only` and `model_version` options are mutually exclusive.")
    }

    if reviewed_only && dataset.is_none() {
        bail!("Cannot get reviewed comments when `dataset` is not provided.")
    }

    if include_predictions.unwrap_or_default() && dataset.is_none() {
        bail!("Cannot get predictions when `dataset` is not provided.")
    }

    if label_filter.is_some() && dataset.is_none() {
        bail!("Cannot use a label filter when `dataset` is not provided.")
    }

    if label_filter.is_some() && reviewed_only {
        bail!("The `reviewed_only` and `label_filter` options are mutually exclusive.")
    }

    if label_filter.is_some() && model_version.is_some() {
        bail!("The `label_filter` and `model_version` options are mutually exclusive.")
    }

    if (user_property_filter.is_some() || *interative_property_filter) && dataset.is_none() {
        bail!("Cannot use a property filter when `dataset` is not provided.")
    }

    if (user_property_filter.is_some() || *interative_property_filter) && reviewed_only {
        bail!("The `reviewed_only` and `property_filter` options are mutually exclusive.")
    }

    if (user_property_filter.is_some() || *interative_property_filter)
        && user_property_filter.is_some()
    {
        bail!("The `interative_property_filter` and `property_filter` options are mutually exclusive.")
    }

    if (senders.is_some() || recipients.is_some()) && dataset.is_none() {
        bail!("Cannot filter on `senders` or `recipients` when `dataset` is not provided")
    }

    let file = match path {
        Some(path) => Some(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?,
        ),
        None => None,
    };

    let mut label_attribute_filter: Option<AttributeFilter> = None;
    if let (Some(dataset_id), Some(filter)) = (dataset, label_filter) {
        label_attribute_filter = get_label_attribute_filter(client, dataset_id.clone(), filter)?;
        // Exit early if no labels match label filter
        if label_attribute_filter.is_none() {
            return Ok(());
        }
    }

    let user_properties_filter = if let Some(filter) = user_property_filter {
        Some(filter.0.clone())
    } else if *interative_property_filter {
        Some(get_user_properties_filter_interactively(
            client,
            dataset.clone().context("Could not unwrap dataset")?,
        )?)
    } else {
        None
    };

    let messages_filter = MessagesFilter {
        from: senders.as_ref().map(|senders| {
            PropertyFilterKind::OneOf(
                senders
                    .iter()
                    .map(|sender| PropertyValue::String(sender.to_owned()))
                    .collect(),
            )
        }),
        to: recipients.as_ref().map(|recipients| {
            PropertyFilterKind::OneOf(
                recipients
                    .iter()
                    .map(|recipient| PropertyValue::String(recipient.to_owned()))
                    .collect(),
            )
        }),
    };

    let download_options = CommentDownloadOptions {
        dataset_identifier: dataset.clone(),
        include_predictions: include_predictions.unwrap_or(false),
        model_version: *model_version,
        reviewed_only,
        timerange: CommentsIterTimerange {
            from: *from_timestamp,
            to: *to_timestamp,
        },
        show_progress: !no_progress,
        label_attribute_filter,
        user_properties_filter,
        messages_filter: Some(messages_filter),
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

fn get_label_attribute_filter(
    client: &Client,
    dataset_id: DatasetIdentifier,
    filter: &Regex,
) -> Result<Option<AttributeFilter>> {
    let dataset = client.get_dataset(dataset_id)?;

    let label_names: Vec<LabelName> = dataset
        .label_defs
        .into_iter()
        .filter(|label_def| filter.is_match(&label_def.name.0))
        .map(|label_def| label_def.name)
        .collect();

    if label_names.is_empty() {
        info!("No label names matching the filter '{}'", filter);
        Ok(None)
    } else {
        info!(
            "Filtering on label(s):\n- {}",
            label_names
                .iter()
                .map(|label_name| label_name.0.as_str())
                .collect::<Vec<_>>()
                .join("\n- ")
        );
        Ok(Some(AttributeFilter {
            attribute: Attribute::Labels,
            filter: AttributeFilterEnum::StringAnyOf {
                any_of: label_names,
            },
        }))
    }
}

struct CommentDownloadOptions {
    dataset_identifier: Option<DatasetIdentifier>,
    include_predictions: bool,
    model_version: Option<u32>,
    reviewed_only: bool,
    timerange: CommentsIterTimerange,
    show_progress: bool,
    label_attribute_filter: Option<AttributeFilter>,
    user_properties_filter: Option<UserPropertiesFilter>,
    messages_filter: Option<MessagesFilter>,
}

impl CommentDownloadOptions {
    fn get_attribute_filters(&self) -> Vec<AttributeFilter> {
        let mut filters: Vec<AttributeFilter> = Vec::new();

        if let Some(label_attribute_filter) = &self.label_attribute_filter {
            filters.push(label_attribute_filter.clone());
        }

        filters
    }
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
        let comment_filter = CommentFilter {
            timestamp: Some(CommentTimestampFilter {
                minimum: options.timerange.from,
                maximum: options.timerange.to,
            }),
            sources: vec![source.id.clone()],
            reviewed: if options.reviewed_only {
                Some(ReviewedFilterEnum::OnlyReviewed)
            } else {
                None
            },
            user_properties: options.user_properties_filter.clone(),
            messages: options.messages_filter.clone(),
        };

        Ok(get_comments_progress_bar(
            if let Some(dataset_name) = dataset_name {
                *client
                    .get_dataset_statistics(
                        dataset_name,
                        &DatasetStatisticsRequestParams {
                            comment_filter,
                            attribute_filters: options.get_attribute_filters(),
                            ..Default::default()
                        },
                    )
                    .context("Operation to get dataset comment count has failed..")?
                    .num_comments as u64
            } else {
                *client
                    .get_source_statistics(
                        &source.full_name(),
                        &SourceStatisticsRequestParams { comment_filter },
                    )
                    .context("Operation to get source comment count has failed..")?
                    .num_comments as u64
            },
            &statistics,
            dataset_name.is_some(),
        ))
    };

    if let Some(dataset_identifier) = &options.dataset_identifier {
        let dataset = client
            .get_dataset(dataset_identifier.clone())
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
                options.model_version,
                writer,
                &options,
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
                        label_properties: None,
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

const DEFAULT_QUERY_PAGE_SIZE: usize = 128;

#[allow(clippy::too_many_arguments)]
fn get_comments_from_uids(
    client: &Client,
    dataset_name: DatasetFullName,
    source: Source,
    statistics: &Arc<Statistics>,
    include_predictions: bool,
    model_version: Option<u32>,
    mut writer: impl Write,
    options: &CommentDownloadOptions,
) -> Result<()> {
    let mut params = QueryRequestParams {
        attribute_filters: options.get_attribute_filters(),
        continuation: None,
        filter: CommentFilter {
            reviewed: None,
            timestamp: Some(CommentTimestampFilter {
                minimum: options.timerange.from,
                maximum: options.timerange.to,
            }),
            user_properties: options.user_properties_filter.clone(),
            sources: vec![source.id],
            messages: options.messages_filter.clone(),
        },
        limit: DEFAULT_QUERY_PAGE_SIZE,
        order: OrderEnum::Recent,
    };

    client
        .get_dataset_query_iter(&dataset_name, &mut params)
        .try_for_each(|page| {
            let page = page.context("Operation to get comments has failed.")?;
            if page.is_empty() {
                return Ok(());
            }

            statistics.add_comments(page.len());

            if let Some(model_version) = &model_version {
                let predictions = client
                    .get_comment_predictions(
                        &dataset_name,
                        &ModelVersion(*model_version),
                        page.iter().map(|comment| &comment.comment.uid),
                    )
                    .context("Operation to get predictions has failed.")?;
                // since predict-comments endpoint doesn't return some fields,
                // they are set to None or [] here
                let comments =
                    page.into_iter()
                        .zip(predictions.into_iter())
                        .map(|(comment, prediction)| AnnotatedComment {
                            comment: comment.comment,
                            labelling: Some(vec![Labelling {
                                group: DEFAULT_LABEL_GROUP_NAME.clone(),
                                assigned: Vec::new(),
                                dismissed: Vec::new(),
                                predicted: prediction.labels.map(|auto_threshold_labels| {
                                    auto_threshold_labels
                                        .iter()
                                        .map(|auto_threshold_label| PredictedLabel {
                                            name: PredictedLabelName::String(LabelName(
                                                auto_threshold_label.name.join(" > "),
                                            )),
                                            sentiment: None,
                                            probability: auto_threshold_label.probability,
                                            auto_thresholds: Some(
                                                auto_threshold_label.auto_thresholds.to_vec(),
                                            ),
                                        })
                                        .collect()
                                }),
                            }]),
                            entities: Some(Entities {
                                assigned: Vec::new(),
                                dismissed: Vec::new(),
                                predicted: prediction.entities,
                            }),
                            thread_properties: None,
                            moon_forms: None,
                            label_properties: None,
                        });
                print_resources_as_json(comments, &mut writer)
            } else {
                let comments = page.into_iter().map(|mut annotated_comment| {
                    if !include_predictions {
                        annotated_comment = annotated_comment.without_predictions();
                    }
                    if annotated_comment.has_annotations() {
                        statistics.add_annotated(1);
                    }
                    annotated_comment
                });
                print_resources_as_json(comments, &mut writer)
            }
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
