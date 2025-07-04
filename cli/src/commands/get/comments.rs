use anyhow::{anyhow, bail, Context, Error, Result};

use chrono::{DateTime, Utc};
use colored::Colorize;
use dialoguer::{Input, MultiSelect, Select};
use log::info;
use ordered_float::NotNan;
use rand::Rng;
use regex::Regex;
use reinfer_client::{
    resources::{
        comment::{
            CommentTimestampFilter, MessagesFilter, PropertyFilter, ReviewedFilterEnum,
            UserPropertiesFilter,
        },
        dataset::{
            Attribute, AttributeFilter, AttributeFilterEnum, OrderEnum, QueryRequestParams,
            StatisticsRequestParams as DatasetStatisticsRequestParams, Summary,
        },
        source::StatisticsRequestParams as SourceStatisticsRequestParams,
    },
    AnnotatedComment, Client, Comment, CommentFilter, CommentId, CommentPredictionsThreshold,
    CommentsIterTimerange, DatasetFullName, DatasetIdentifier, Entities, HasAnnotations, Labelling,
    ModelVersion, PredictedLabel, PropertyValue, Source, SourceIdentifier,
    DEFAULT_LABEL_GROUP_NAME,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{create_dir, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

use crate::{
    commands::LocalAttachmentPath,
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

    #[structopt(long = "attachment-types")]
    /// The list of attachment types to filter to
    attachment_type_filters: Vec<String>,

    #[structopt(long = "attachments")]
    /// Save attachment content for each comment
    include_attachment_content: Option<bool>,

    #[structopt(long = "--only-with-attachments")]
    /// Whether to only return comments with attachment metadata
    only_with_attachments: Option<bool>,

    #[structopt(long = "--shuffle")]
    /// Whether to return comments in a random order
    shuffle: Option<bool>,

    #[structopt(long = "--stop-after")]
    /// Stop downloading comments after X comments (stops in following batch)
    stop_after: Option<usize>,
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

const PROPERTY_VALUE_COUNT_CIRCUIT_BREAKER: usize = 256;

pub fn get_user_properties_filter_interactively(summary: &Summary) -> Result<UserPropertiesFilter> {
    let string_user_property_selections = MultiSelect::new()
        .with_prompt("Select which string user properties you want to set up filters for")
        .items(
            &summary
                .user_properties
                .string
                .iter()
                .map(|p| p.full_name.clone())
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let number_user_property_selections = MultiSelect::new()
        .with_prompt("Select which string user properties you want to set up filters for")
        .items(
            &summary
                .user_properties
                .number
                .iter()
                .map(|p| p.full_name.clone())
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let string_property_filters: HashMap<String, PropertyFilter> = string_user_property_selections
        .iter()
        .map(|selection| {
            let property = &summary.user_properties.string[*selection];

            let actions = ["Inclusion", "Exclusion", "Both"];
            let action_selection = Select::new()
                .with_prompt(format!(
                    "How do you want to filter the property \"{}\"",
                    property.full_name
                ))
                .items(&actions)
                .interact()
                .expect("Could not get property filter action selection from user");

            let get_property_value_selections =
                |prompt: String, possible_values: &mut Vec<String>| -> Result<Vec<PropertyValue>> {
                    if possible_values.len() < PROPERTY_VALUE_COUNT_CIRCUIT_BREAKER {
                        let selections = MultiSelect::new()
                            .with_prompt(prompt)
                            .items(possible_values)
                            .interact()?;

                        let values = selections
                            .iter()
                            .map(|selection| {
                                let selection_value = &possible_values[*selection];
                                PropertyValue::String(selection_value.clone())
                            })
                            .collect();

                        for selection in &selections {
                            possible_values.remove(*selection);
                        }
                        Ok(values)
                    } else {
                        let comma_sep_values: String = Input::new()
                            .with_prompt("Please enter your values seperated by comma")
                            .interact_text()?;

                        Ok(comma_sep_values
                            .to_lowercase()
                            .split(',')
                            .map(|value| PropertyValue::String(value.trim().to_string()))
                            .collect())
                    }
                };

            let mut filter: PropertyFilter = Default::default();
            let mut possible_values =
                get_possible_values_for_string_property(summary, &property.full_name)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Could not get possible values for user property \"{}\"",
                            property.full_name
                        )
                    });
            match actions[action_selection] {
                "Inclusion" => {
                    filter.one_of = get_property_value_selections(
                        format!(
                            "What values do you want to include for the property \"{}\"",
                            property.full_name
                        ),
                        &mut possible_values,
                    )
                    .expect("Could not get property selection from user")
                }
                "Exclusion" => {
                    filter.not_one_of = get_property_value_selections(
                        format!(
                            "What values do you want to exclude for the property \"{}\"",
                            property.full_name
                        ),
                        &mut possible_values,
                    )
                    .expect("Could not get property selection from user")
                }
                "Both" => {
                    let one_ofs = get_property_value_selections(
                        format!(
                            "What values do you want to include for the property \"{}\"",
                            property.full_name
                        ),
                        &mut possible_values,
                    )
                    .expect("Could not get property selection from user");

                    let not_one_ofs = get_property_value_selections(
                        format!(
                            "What values do you want to exclude for the property \"{}\"",
                            property.full_name
                        ),
                        &mut possible_values,
                    )
                    .expect("Could not get property selection from user");

                    filter.one_of = one_ofs;
                    filter.not_one_of = not_one_ofs;
                }
                _ => {
                    panic!("Invalid Action Selection")
                }
            }

            (property.full_name.clone(), filter)
        })
        .collect();

    let number_property_filters: HashMap<String, PropertyFilter> = number_user_property_selections
        .iter()
        .map(|selection| {
            let property = &summary.user_properties.number[*selection];

            let min = Input::new()
                .with_prompt(format!(
                    "What is the minimum value that you want to filter \"{}\" for?",
                    property.full_name
                ))
                .validate_with(|input: &String| match input.trim().parse::<NotNan<f64>>() {
                    Ok(number) => {
                        if number >= NotNan::new(0.0).unwrap() {
                            Ok(())
                        } else {
                            Err("Please enter a number greater than 0")
                        }
                    }
                    Err(_) => Err("Please enter a valid number"),
                })
                .interact()
                .expect("Could not get input from user");

            let max = Input::new()
                .with_prompt(format!(
                    "What is the maximum value that you want to filter \"{}\" for?",
                    property.full_name
                ))
                .validate_with(|input: &String| match input.trim().parse::<NotNan<f64>>() {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Please enter a valid number"),
                })
                .interact()
                .expect("Could not get input from user");

            (
                property.full_name.clone(),
                PropertyFilter {
                    minimum: Some(min.trim().parse::<NotNan<f64>>().unwrap()),
                    maximum: Some(max.trim().parse::<NotNan<f64>>().unwrap()),
                    ..Default::default()
                },
            )
        })
        .collect();

    let property_filters: HashMap<String, PropertyFilter> = string_property_filters
        .into_iter()
        .chain(number_property_filters)
        .collect();

    Ok(UserPropertiesFilter(property_filters))
}

fn get_possible_values_for_string_property(
    dataset_summary: &Summary,
    property_name: &String,
) -> Result<Vec<String>> {
    Ok(dataset_summary
        .user_properties
        .string
        .iter()
        .find(|p| &p.full_name == property_name)
        .context("Could not get possible values for property with the name {property_name}")?
        .values
        .iter()
        .map(|v| v.value.clone())
        .collect())
}

#[derive(Default)]
struct OutputLocations {
    jsonl_file: Option<BufWriter<std::fs::File>>,
    attachments_dir: Option<PathBuf>,
}

fn get_output_locations(path: &Option<PathBuf>, attachments: bool) -> Result<OutputLocations> {
    if let Some(path) = path {
        let jsonl_file = Some(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?,
        );

        let attachments_dir = if attachments {
            let attachments_dir = path
                .parent()
                .context("Could not get attachments directory")?
                .join(format!(
                    "{0}.attachments",
                    path.file_name()
                        .context("Could not get output file name")?
                        .to_string_lossy()
                ));

            if !attachments_dir.exists() {
                create_dir(&attachments_dir)?;
            }
            Some(attachments_dir)
        } else {
            None
        };

        Ok(OutputLocations {
            jsonl_file,
            attachments_dir,
        })
    } else {
        Ok(OutputLocations::default())
    }
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
        attachment_type_filters,
        property_filter: user_property_filter,
        interactive_property_filter: interative_property_filter,
        recipients,
        senders,
        include_attachment_content,
        only_with_attachments,
        shuffle,
        stop_after,
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

    if (!attachment_type_filters.is_empty() | only_with_attachments.is_some()) && dataset.is_none()
    {
        bail!("Cannot use a attachment type filter when `dataset` is not provided.")
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

    if user_property_filter.is_some() && *interative_property_filter {
        bail!("The `interative_property_filter` and `property_filter` options are mutually exclusive.")
    }

    if (senders.is_some() || recipients.is_some()) && dataset.is_none() {
        bail!("Cannot filter on `senders` or `recipients` when `dataset` is not provided")
    }

    if path.is_none() && include_attachment_content.is_some() {
        bail!("Cannot include attachment content when no file is provided")
    }

    if shuffle.is_some() && dataset.is_none() {
        bail!("Cannot shuffle data when dataset is not provided")
    }

    let OutputLocations {
        jsonl_file,
        attachments_dir,
    } = get_output_locations(path, include_attachment_content.unwrap_or_default())?;

    let mut label_attribute_filter: Option<AttributeFilter> = None;
    if let (Some(dataset_id), Some(filter)) = (dataset, label_filter) {
        label_attribute_filter = get_label_attribute_filter(client, dataset_id.clone(), filter)?;
        // Exit early if no labels match label filter
        if label_attribute_filter.is_none() {
            return Ok(());
        }
    }

    let mut attachment_property_types_filter: Option<AttributeFilter> = None;

    if !attachment_type_filters.is_empty() {
        attachment_property_types_filter = Some(AttributeFilter {
            attribute: Attribute::AttachmentPropertyTypes,
            filter: AttributeFilterEnum::StringAnyOf {
                any_of: attachment_type_filters.to_vec(),
            },
        });
    }

    let mut only_with_attachments_filter: Option<AttributeFilter> = None;
    if only_with_attachments.unwrap_or_default() {
        only_with_attachments_filter = Some(AttributeFilter {
            attribute: Attribute::AttachmentPropertyNumAttachments,
            filter: AttributeFilterEnum::NumberRange {
                minimum: Some(1),
                maximum: None,
            },
        });
    }

    let user_properties_filter = if let Some(filter) = user_property_filter {
        Some(filter.0.clone())
    } else if *interative_property_filter {
        let dataset = client.get_dataset(dataset.clone().context("Could not get dataset")?)?;
        let summary_response = client.dataset_summary(&dataset.full_name(), &Default::default())?;
        Some(get_user_properties_filter_interactively(
            &summary_response.summary,
        )?)
    } else {
        None
    };

    let messages_filter = MessagesFilter {
        from: senders.as_ref().map(|senders| {
            PropertyFilter::new(
                senders
                    .iter()
                    .map(|sender| PropertyValue::String(sender.to_owned()))
                    .collect(),
                Vec::new(),
                Vec::new(),
            )
        }),
        to: recipients.as_ref().map(|recipients| {
            PropertyFilter::new(
                recipients
                    .iter()
                    .map(|recipient| PropertyValue::String(recipient.to_owned()))
                    .collect(),
                Vec::new(),
                Vec::new(),
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
        attachment_property_types_filter,
        messages_filter: Some(messages_filter),
        attachments_dir,
        only_with_attachments_filter,
        shuffle: shuffle.unwrap_or(false),
        stop_after: *stop_after,
    };

    if let Some(file) = jsonl_file {
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

    let label_names: Vec<String> = dataset
        .label_defs
        .into_iter()
        .filter(|label_def| filter.is_match(&label_def.name.0))
        .map(|label_def| label_def.name.0)
        .collect();

    if label_names.is_empty() {
        info!("No label names matching the filter '{filter}'");
        Ok(None)
    } else {
        info!("Filtering on label(s):\n- {}", label_names.join("\n- "));
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
    attachment_property_types_filter: Option<AttributeFilter>,
    user_properties_filter: Option<UserPropertiesFilter>,
    messages_filter: Option<MessagesFilter>,
    attachments_dir: Option<PathBuf>,
    only_with_attachments_filter: Option<AttributeFilter>,
    shuffle: bool,
    stop_after: Option<usize>,
}

impl CommentDownloadOptions {
    fn get_attribute_filters(&self) -> Vec<AttributeFilter> {
        let mut filters: Vec<AttributeFilter> = Vec::new();

        if let Some(label_attribute_filter) = &self.label_attribute_filter {
            filters.push(label_attribute_filter.clone());
        }

        if let Some(attachment_types_attribute_filter) = &self.attachment_property_types_filter {
            filters.push(attachment_types_attribute_filter.clone())
        }

        if let Some(only_with_attachments_filter) = &self.only_with_attachments_filter {
            filters.push(only_with_attachments_filter.clone())
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

        let total_comments = if let Some(dataset_name) = dataset_name {
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
        };

        Ok(get_comments_progress_bar(
            if let Some(stop_after) = options.stop_after {
                std::cmp::min(stop_after as u64, total_comments)
            } else {
                total_comments
            },
            &statistics,
            dataset_name.is_some(),
            options.attachments_dir.is_some(),
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
                writer,
                options,
            )?;
        } else {
            get_comments_from_uids(client, dataset_name, source, &statistics, writer, &options)?;
        }
    } else {
        let _progress = if options.show_progress {
            Some(make_progress(None)?)
        } else {
            None
        };
        for page in client.get_comments_iter(&source.full_name(), None, options.timerange) {
            let page = page.context("Operation to get comments has failed.")?;

            if options
                .stop_after
                .is_some_and(|stop_after| statistics.num_downloaded() >= stop_after)
            {
                break;
            }

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
            )?;
        }
    }
    log::info!(
        "Successfully downloaded {} comments [{} annotated].",
        statistics.num_downloaded(),
        statistics.num_annotated(),
    );
    Ok(())
}

pub const DEFAULT_QUERY_PAGE_SIZE: usize = 512;

#[allow(clippy::too_many_arguments)]
fn get_comments_from_uids(
    client: &Client,
    dataset_name: DatasetFullName,
    source: Source,
    statistics: &Arc<Statistics>,
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
        limit: Some(DEFAULT_QUERY_PAGE_SIZE),
        order: if options.shuffle {
            OrderEnum::Sample {
                seed: rand::thread_rng().gen_range(0..2_i64.pow(31) - 1) as usize,
            }
        } else {
            OrderEnum::Recent
        },
    };

    for page in client.get_dataset_query_iter(&dataset_name, &mut params) {
        let page = page.context("Operation to get comments has failed.")?;
        if page.is_empty() {
            return Ok(());
        }

        if options
            .stop_after
            .is_some_and(|stop_after| statistics.num_downloaded() >= stop_after)
        {
            break;
        }

        statistics.add_comments(page.len());

        if let Some(model_version) = &options.model_version {
            let predictions = client
                .get_comment_predictions(
                    &dataset_name,
                    &ModelVersion(*model_version),
                    page.iter().map(|comment| &comment.comment.uid),
                    Some(CommentPredictionsThreshold::Auto),
                    None,
                )
                .context("Operation to get predictions has failed.")?;
            // since predict-comments endpoint doesn't return some fields,
            // they are set to None or [] here
            let comments: Vec<_> = page
                .into_iter()
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
                                    name: auto_threshold_label.name.clone(),
                                    sentiment: None,
                                    probability: auto_threshold_label.probability,
                                    auto_thresholds: Some(
                                        auto_threshold_label
                                            .auto_thresholds
                                            .clone()
                                            .expect("Could not get auto thresholds")
                                            .to_vec(),
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
                })
                .collect();

            if let Some(attachments_dir) = &options.attachments_dir {
                comments.iter().try_for_each(|comment| -> Result<()> {
                    download_comment_attachments(
                        client,
                        attachments_dir,
                        &comment.comment,
                        statistics,
                    )
                })?;
            }
            print_resources_as_json(comments, &mut writer)?;
        } else {
            let comments: Vec<_> = page
                .into_iter()
                .map(|mut annotated_comment| {
                    if !options.include_predictions {
                        annotated_comment = annotated_comment.without_predictions();
                    }
                    if annotated_comment.has_annotations() {
                        statistics.add_annotated(1);
                    }
                    annotated_comment
                })
                .collect();
            if let Some(attachments_dir) = &options.attachments_dir {
                comments.iter().try_for_each(|comment| -> Result<()> {
                    download_comment_attachments(
                        client,
                        attachments_dir,
                        &comment.comment,
                        statistics,
                    )
                })?;
            }

            print_resources_as_json(comments, &mut writer)?;
        }
    }
    Ok(())
}

fn download_comment_attachments(
    client: &Client,
    attachments_dir: &Path,
    comment: &Comment,
    statistics: &Arc<Statistics>,
) -> Result<()> {
    comment
        .attachments
        .iter()
        .enumerate()
        .try_for_each(|(idx, attachment)| -> Result<()> {
            if let Some(attachment_reference) = &attachment.attachment_reference {
                let local_attachment = LocalAttachmentPath {
                    index: idx,
                    name: attachment.name.clone(),
                    parent_dir: attachments_dir.join(&comment.id.0),
                };

                if !local_attachment.exists() {
                    let attachment_buf = client.get_attachment(attachment_reference)?;

                    if local_attachment.write(attachment_buf)? {
                        statistics.add_attachments(1);
                    };
                }
            }
            Ok(())
        })?;
    Ok(())
}

fn get_reviewed_comments_in_bulk(
    client: &Client,
    dataset_name: DatasetFullName,
    source: Source,
    statistics: &Arc<Statistics>,
    mut writer: impl Write,
    options: CommentDownloadOptions,
) -> Result<()> {
    for page in
        client.get_labellings_iter(&dataset_name, &source.id, options.include_predictions, None)
    {
        let page = page.context("Operation to get labellings has failed.")?;

        if options
            .stop_after
            .is_some_and(|stop_after| statistics.num_downloaded() >= stop_after)
        {
            break;
        }

        statistics.add_comments(page.len());
        statistics.add_annotated(page.len());

        if let Some(attachments_dir) = &options.attachments_dir {
            page.iter().try_for_each(|comment| -> Result<()> {
                download_comment_attachments(client, attachments_dir, &comment.comment, statistics)
            })?;
        }

        let comments = page.into_iter().map(|comment| {
            if !options.include_predictions {
                comment.without_predictions()
            } else {
                comment
            }
        });

        print_resources_as_json(comments, &mut writer)?;
    }
    Ok(())
}

#[derive(Debug)]
pub struct Statistics {
    downloaded: AtomicUsize,
    annotated: AtomicUsize,
    attachments: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            downloaded: AtomicUsize::new(0),
            annotated: AtomicUsize::new(0),
            attachments: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_comments(&self, num_downloaded: usize) {
        self.downloaded.fetch_add(num_downloaded, Ordering::SeqCst);
    }

    #[inline]
    fn add_attachments(&self, num_downloaded: usize) {
        self.attachments.fetch_add(num_downloaded, Ordering::SeqCst);
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
    fn num_attachments(&self) -> usize {
        self.attachments.load(Ordering::SeqCst)
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
    show_attachments: bool,
) -> Progress {
    Progress::new(
        move |statistics| {
            let num_downloaded = statistics.num_downloaded();
            let num_annotated = statistics.num_annotated();
            let num_attachments = statistics.num_attachments();
            (
                num_downloaded as u64,
                format!(
                    "{} {}{}{}",
                    num_downloaded.to_string().bold(),
                    "comments".dimmed(),
                    if show_annotated {
                        format!(" [{} {}]", num_annotated, "annotated".dimmed())
                    } else {
                        "".into()
                    },
                    if show_attachments {
                        format!(" [{} {}]", num_attachments, "attachments".dimmed())
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
