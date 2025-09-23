use anyhow::{anyhow, bail, Context, Error, Result};

use chrono::{DateTime, Utc};
use colored::Colorize;
use dialoguer::{Input, MultiSelect, Select};
use log::info;
use ordered_float::NotNan;
use rand::Rng;
use regex::Regex;
use openapi::{
    apis::{
        configuration::Configuration,
        comments_api::{get_comment, get_source_comments, query_comments},
        sources_api::{get_source, get_source_by_id, get_source_statistics},
        attachments_api::get_attachment,
        datasets_api::{get_dataset, get_dataset_summary, get_dataset_statistics, get_labellings},
        models_api::get_comment_predictions,
    },
    models::{
        AttributeFilterAttribute, AnnotatedComment, Dataset, Source, AttributeFilter, MessageFilter, CommentFilter, TimestampRangeFilter, StringArrayFilter, FullParticipantFilter, Reviewed, GetDatasetSummaryRequest, GetDatasetStatisticsRequest, GetSourceStatisticsRequest, Comment, GetSourceCommentsResponse, QueryCommentsRequest, Order, QueryCommentsOrderSample, Kind, QueryCommentsResponse, GetCommentPredictionsRequest, CommentPrediction, PredictedLabel, PredictedEntity, Labelling, Entities
    }
};
use crate::utils::{
    CommentId, SourceIdentifier, DatasetIdentifier, AttributeFilterEnum, CommentsIterTimerange,
};

// Removed custom CommentsIter - now using direct API calls with built-in pagination

use crate::utils::user_properties_filter::{UserPropertiesFilter, PropertyFilter, PropertyValue};
use openapi::models::DatasetSummary;
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

    #[structopt(long = "attachments")]
    /// Save attachment content for each comment
    include_attachment_content: Option<bool>,
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

pub fn get_single(config: &Configuration, args: &GetSingleCommentArgs) -> Result<()> {
    let GetSingleCommentArgs {
        source,
        comment_id,
        path,
        include_attachment_content,
    } = args;

    if path.is_none() && include_attachment_content.is_some() {
        bail!("Cannot include attachment content when no file is provided")
    }

    let include_attachment_content = include_attachment_content.unwrap_or_default();

    let OutputLocations {
        jsonl_file: file,
        attachments_dir,
    } = get_output_locations(path, include_attachment_content)?;

    let stdout = io::stdout();
    let mut writer: Box<dyn Write> = file.unwrap_or_else(|| Box::new(stdout.lock()));
    let source = match &source {
        SourceIdentifier::FullName(full_name) => {
            let response = get_source(config, full_name.owner(), full_name.name())
                .with_context(|| format!("Unable to get source {}", source))?;
            *response.source
        }
        SourceIdentifier::Id(id) => {
            let response = get_source_by_id(config, id)
                .with_context(|| format!("Unable to get source {}", source))?;
            *response.source
        }
    };
    
    let comment_response = get_comment(config, &source.owner, &source.name, comment_id.as_ref(), None)
        .with_context(|| format!("Unable to get comment {}", comment_id))?;
    let comment = *comment_response.comment;

    if let Some(attachments_dir) = attachments_dir {
        download_comment_attachments(config, &attachments_dir, &comment, None)?;
    }
    print_resources_as_json(
        std::iter::once(AnnotatedComment {
            comment: Box::new(comment),
            labelling: Vec::new(), // Required field
            reviewable_blocks: None,
            entities: None,
            thread_properties: None,
            trigger_exceptions: None,
            label_properties: None,
            moon_forms: None,
            extractions: None,
            highlights: None,
            diagnostics: None,
            model_version: None,
        }),
        &mut writer,
    )
}

const PROPERTY_VALUE_COUNT_CIRCUIT_BREAKER: usize = 256;

pub fn get_user_properties_filter_interactively(summary: &DatasetSummary) -> Result<UserPropertiesFilter> {
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
    dataset_summary: &DatasetSummary,
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
    jsonl_file: Option<Box<dyn Write>>,
    attachments_dir: Option<PathBuf>,
}

fn get_output_locations(path: &Option<PathBuf>, attachments: bool) -> Result<OutputLocations> {
    if let Some(path) = path {
        let jsonl_file: Option<Box<dyn Write>> = Some(Box::new(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?,
        ));

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

pub fn get_many(config: &Configuration, args: &GetManyCommentsArgs) -> Result<()> {
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
        label_attribute_filter = get_label_attribute_filter(config, dataset_id.clone(), filter)?;
        // Exit early if no labels match label filter
        if label_attribute_filter.is_none() {
            return Ok(());
        }
    }

    let mut attachment_property_types_filter: Option<AttributeFilter> = None;

    if !attachment_type_filters.is_empty() {
        attachment_property_types_filter = Some(AttributeFilter {
            attribute: Box::new(AttributeFilterAttribute::AttachmentPropertyTypes),
            filter: Box::new(AttributeFilterEnum::StringAnyOf {
                any_of: attachment_type_filters.to_vec(),
            }),
        });
    }

    let mut only_with_attachments_filter: Option<AttributeFilter> = None;
    if only_with_attachments.unwrap_or_default() {
        only_with_attachments_filter = Some(AttributeFilter {
            attribute: Box::new(AttributeFilterAttribute::AttachmentPropertyNumAttachments),
            filter: Box::new(AttributeFilterEnum::NumberRange {
                minimum: Some(1),
                maximum: None,
            }),
        });
    }

    let user_properties_filter = if let Some(filter) = user_property_filter {
        Some(filter.0.clone())
    } else if *interative_property_filter {
        if let Some(dataset_id) = dataset {
            // Get dataset by owner and name
            let dataset_response = get_dataset(config, &dataset_id.owner, &dataset_id.name)
                .context("Could not get dataset")?;
            let dataset = dataset_response.dataset;
            
            // Get dataset summary
            let summary_request = GetDatasetSummaryRequest::new();
            let summary_response = get_dataset_summary(config, &dataset.owner, &dataset.name, summary_request)
                .context("Could not get dataset summary")?;
            
            Some(get_user_properties_filter_interactively(
                &summary_response.summary,
            )?)
        } else {
            bail!("Dataset identifier required for interactive property filter")
        }
    } else {
        None
    };

    let messages_filter = MessageFilter {
        from: senders.as_ref().map(|senders| {
            Box::new(FullParticipantFilter {
                one_of: Some(senders.clone()),
                not_one_of: None,
                domain_one_of: None,
                domain_not_one_of: None,
                organisation_one_of: None,
                organisation_not_one_of: None,
            })
        }),
        to: recipients.as_ref().map(|recipients| {
            Box::new(StringArrayFilter {
                one_of: Some(recipients.clone()),
                not_one_of: None,
                include_missing: None,
            })
        }),
        recipient: None,
        cc: None,
        bcc: None,
    };

    let download_options = CommentDownloadOptions {
        dataset_identifier: dataset.clone(),
        include_predictions: include_predictions.unwrap_or(false),
        model_version: *model_version,
        reviewed_only,
        timerange: CommentsIterTimerange::new(*from_timestamp, *to_timestamp),
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
        download_comments(config, source.clone(), file, download_options)
    } else {
        download_comments(
            config,
            source.clone(),
            io::stdout().lock(),
            download_options,
        )
    }
}

fn get_label_attribute_filter(
    config: &Configuration,
    dataset_id: DatasetIdentifier,
    filter: &Regex,
) -> Result<Option<AttributeFilter>> {
    let dataset_response = get_dataset(config, &dataset_id.owner, &dataset_id.name)
        .context("Could not get dataset")?;
    let dataset = dataset_response.dataset;

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
            attribute: Box::new(AttributeFilterAttribute::Labels),
            filter: Box::new(AttributeFilterEnum::StringAnyOf {
                any_of: label_names,
            }),
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
    messages_filter: Option<MessageFilter>,
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
    config: &Configuration,
    source_identifier: SourceIdentifier,
    mut writer: impl Write,
    options: CommentDownloadOptions,
) -> Result<()> {
    let source = match source_identifier {
        SourceIdentifier::FullName(full_name) => {
            let response = get_source(config, full_name.owner(), full_name.name())
                .context("Operation to get source has failed.")?;
            *response.source
        }
        SourceIdentifier::Id(id) => {
            let response = get_source_by_id(config, &id)
                .context("Operation to get source has failed.")?;
            *response.source
        }
    };
    let statistics = Arc::new(Statistics::new());

    let make_progress = |dataset: Option<&Dataset>| -> Result<Progress> {
        let comment_filter = CommentFilter {
            entities: None,
            thread_properties: None,
            timestamp: Some(Box::new(TimestampRangeFilter {
                minimum: options.timerange.from.map(|dt| dt.to_rfc3339()),
                maximum: options.timerange.to.map(|dt| dt.to_rfc3339()),
            })),
            reviewed: if options.reviewed_only {
                Some(Reviewed::Reviewed)
            } else {
                None
            },
            sources: Some(vec![source.id.clone()]),
            messages: options.messages_filter.clone(),
            user_properties: options.user_properties_filter.clone(),
            trigger_exceptions: None,
            annotations: None,
        };

        let total_comments = if let Some(dataset) = dataset {
            // Get dataset statistics
            let stats_request = GetDatasetStatisticsRequest {
                comment_filter: Some(comment_filter),
                attribute_filters: Some(options.get_attribute_filters()),
                by_label_properties: Vec::new(), // Required field
                ..Default::default()
            };
            let stats_response = get_dataset_statistics(config, &dataset.owner, &dataset.name, stats_request)
                .context("Operation to get dataset comment count has failed.")?;
            stats_response.statistics.num_comments as u64
        } else {
            // Get source statistics
            let stats_request = GetSourceStatisticsRequest {
                comment_filter: Some(comment_filter),
            };
            let stats_response = get_source_statistics(config, &source.owner, &source.name, stats_request)
                .context("Operation to get source comment count has failed.")?;
            stats_response.statistics.num_comments as u64
        };

        Ok(get_comments_progress_bar(
            if let Some(stop_after) = options.stop_after {
                std::cmp::min(stop_after as u64, total_comments)
            } else {
                total_comments
            },
            &statistics,
            dataset.is_some(),
            options.attachments_dir.is_some(),
        ))
    };

    if let Some(dataset_identifier) = &options.dataset_identifier {
        let dataset = match dataset_identifier {
            DatasetIdentifier::FullName(full_name) => {
                let response = get_dataset(config, full_name.owner(), full_name.name())
                    .context("Operation to get dataset has failed.")?;
                *response.dataset
            }
            DatasetIdentifier::Id(id) => {
                // For now, we'll need to handle ID-based lookup differently
                // This might require a different API endpoint or we need to get the dataset info first
                bail!("Dataset lookup by ID not yet implemented with OpenAPI")
            }
        };
        
        let _progress = if options.show_progress {
            Some(make_progress(Some(&dataset))?)
        } else {
            None
        };

        if options.reviewed_only {
            get_reviewed_comments_in_bulk(
                config,
                dataset,
                source,
                &statistics,
                writer,
                options,
            )?;
        } else {
            get_comments_from_uids(config, dataset, source, &statistics, writer, &options)?;
        }
    } else {
        let _progress = if options.show_progress {
            Some(make_progress(None)?)
        } else {
            None
        };
        
        // Use direct API calls with built-in pagination instead of custom iterator
        let mut continuation: Option<String> = None;
        loop {
            let response = get_source_comments(
                config,
                &source.owner,
                &source.name,
                continuation.as_deref(),
                Some(256), // Use max page size for efficiency
                options.timerange.from.map(|dt| dt.to_rfc3339().as_str()),
                options.timerange.to.map(|dt| dt.to_rfc3339().as_str()),
                None, // direction
                None, // include_thread_properties
                None, // include_markup
            ).context("Operation to get comments has failed.")?;

            let page = response.comments;
            if page.is_empty() {
                break;
            }

            if options
                .stop_after
                .is_some_and(|stop_after| statistics.num_downloaded() >= stop_after)
            {
                break;
            }

            statistics.add_comments(page.len());

            print_resources_as_json(
                page.into_iter().map(|comment| AnnotatedComment {
                    comment: Box::new(comment),
                    labelling: Vec::new(),
                    reviewable_blocks: None,
                    entities: None,
                    thread_properties: None,
                    trigger_exceptions: None,
                    label_properties: None,
                    moon_forms: None,
                    extractions: None,
                    highlights: None,
                    diagnostics: None,
                    model_version: None,
                }),
                &mut writer,
            )?;

            // Update continuation for next iteration
            continuation = response.continuation;
            if continuation.is_none() {
                break;
            }
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
    config: &Configuration,
    dataset: Dataset,
    source: Source,
    statistics: &Arc<Statistics>,
    mut writer: impl Write,
    options: &CommentDownloadOptions,
) -> Result<()> {
    let mut continuation: Option<String> = None;
    
    loop {
        let order = if options.shuffle {
            Order::Sample(Box::new(QueryCommentsOrderSample::new(
                Kind::Sample,
                rand::thread_rng().gen_range(0..2_i64.pow(31) - 1),
            )))
        } else {
            Order::Recent(Default::default())
        };

        let request = QueryCommentsRequest {
            continuation: continuation.clone(),
            limit: Some(DEFAULT_QUERY_PAGE_SIZE as i32),
            attribute_filters: Some(options.get_attribute_filters()),
            collapse_mode: None,
            filter: Some(CommentFilter {
                entities: None,
                thread_properties: None,
                timestamp: Some(Box::new(TimestampRangeFilter {
                    minimum: options.timerange.from.map(|dt| dt.to_rfc3339()),
                    maximum: options.timerange.to.map(|dt| dt.to_rfc3339()),
                })),
                reviewed: None,
                sources: Some(vec![source.id.clone()]),
                messages: options.messages_filter.clone(),
                user_properties: options.user_properties_filter.clone(),
                trigger_exceptions: None,
                annotations: None,
            }),
            order: Box::new(order),
        };

        let response = query_comments(config, &dataset.owner, &dataset.name, request)
            .context("Operation to get comments has failed.")?;
        
        let page = response.results;
        if page.is_empty() {
            break;
        }

        if options
            .stop_after
            .is_some_and(|stop_after| statistics.num_downloaded() >= stop_after)
        {
            break;
        }

        statistics.add_comments(page.len());
        
        // Update continuation for next iteration
        continuation = response.continuation;
        if continuation.is_none() {
            break;
        }

        if let Some(model_version) = &options.model_version {
            // Get predictions for the comments using the new OpenAPI client
            let uids: Vec<String> = page.iter()
                .map(|comment| comment.comment.uid.clone())
                .collect();
            
            let prediction_request = GetCommentPredictionsRequest::new(uids);
            let predictions_response = get_comment_predictions(
                config,
                &dataset.owner,
                &dataset.name,
                &model_version.to_string(),
                prediction_request,
            ).context("Operation to get predictions has failed.")?;
            
            let predictions = predictions_response.predictions;
            
            // Create a map of uid to prediction for easy lookup
            let mut prediction_map = std::collections::HashMap::new();
            for prediction in predictions {
                prediction_map.insert(prediction.uid.clone(), prediction);
            }
            
            let comments: Vec<_> = page
                .into_iter()
                .map(|comment| {
                    let prediction = prediction_map.get(&comment.comment.uid);
                    AnnotatedComment {
                        comment: comment.comment,
                        labelling: Some(vec![Labelling {
                            group: "default".to_string(),
                            assigned: Vec::new(),
                            dismissed: Vec::new(),
                            predicted: prediction.map(|p| p.labels.clone()),
                        }]),
                        entities: Some(Entities {
                            assigned: Vec::new(),
                            dismissed: Vec::new(),
                            predicted: prediction.and_then(|p| p.entities.clone()),
                        }),
                        reviewable_blocks: None,
                        thread_properties: None,
                        trigger_exceptions: None,
                        label_properties: prediction.and_then(|p| p.label_properties.clone()),
                        moon_forms: None,
                        extractions: None,
                        highlights: None,
                        diagnostics: None,
                        model_version: Some(*model_version),
                    }
                })
                .collect();

            if let Some(attachments_dir) = &options.attachments_dir {
                comments.iter().try_for_each(|comment| -> Result<()> {
                    download_comment_attachments(
                        config,
                        attachments_dir,
                        &comment.comment,
                        Some(statistics),
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
                        config,
                        attachments_dir,
                        &comment.comment,
                        Some(statistics),
                    )
                })?;
            }

            print_resources_as_json(comments, &mut writer)?;
        }
    }
    Ok(())
}

fn download_comment_attachments(
    config: &Configuration,
    attachments_dir: &Path,
    comment: &openapi::models::Comment,
    statistics: Option<&Arc<Statistics>>,
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
                    parent_dir: attachments_dir.join(&comment.id),
                };

                if !local_attachment.exists() {
                    // The generated client downloads to a temporary file and returns the path
                    let temp_file_path = get_attachment(config, attachment_reference)
                        .context("Failed to download attachment")?;
                    
                    // Read the content from the temporary file
                    let attachment_buf = std::fs::read(&temp_file_path)
                        .context("Failed to read downloaded attachment")?;
                    
                    // Clean up the temporary file
                    let _ = std::fs::remove_file(&temp_file_path);

                    if local_attachment.write(attachment_buf)? {
                        if let Some(statistics) = statistics {
                            statistics.add_attachments(1);
                        }
                    };
                }
            }
            Ok(())
        })?;
    Ok(())
}

fn get_reviewed_comments_in_bulk(
    config: &Configuration,
    dataset: Dataset,
    source: Source,
    statistics: &Arc<Statistics>,
    mut writer: impl Write,
    options: CommentDownloadOptions,
) -> Result<()> {
    let mut after: Option<String> = None;
    
    loop {
        let response = get_labellings(config, &dataset.owner, &dataset.name)
            .context("Operation to get labellings has failed.")?;
        
        let page = response.results;

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
                download_comment_attachments(
                    config,
                    attachments_dir,
                    &comment.comment,
                    Some(statistics),
                )
            })?;
        }

        let comments = page.into_iter().map(|mut comment| {
            if !options.include_predictions {
                // Manually remove predictions by filtering out unassigned labels
                comment.labelling = comment.labelling.into_iter()
                    .map(|mut group| {
                        // Keep only assigned labels, remove dismissed and uninformative
                        group.assigned = group.assigned;
                        group.dismissed = Vec::new();
                        group.uninformative = None;
                        group
                    })
                    .collect();
            }
            comment
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
