use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use colored::Colorize;
use dialoguer::{FuzzySelect, Input, MultiSelect};
use log::info;
use ordered_float::NotNan;
use reinfer_client::{
    resources::{
        comment::{CommentTimestampFilter, UserPropertiesFilter},
        dataset::{OrderEnum, QueryRequestParams, StatisticsRequestParams},
    },
    Client, CommentFilter, LabelDef, ModelVersion, Prediction, TriggerLabelThreshold,
};
use structopt::StructOpt;

use crate::{
    commands::get::comments::get_user_properties_filter_interactively,
    printer::Printer,
    progress::{Options as ProgressOptions, Progress},
};

use super::comments::DEFAULT_QUERY_PAGE_SIZE;

#[derive(Debug, StructOpt)]
pub struct GetCustomLabelTrendReportArgs {}

pub fn get(
    client: &Client,
    _args: &GetCustomLabelTrendReportArgs,
    _printer: &Printer,
) -> Result<()> {
    info!("Getting datasets...");
    let datasets = client.get_datasets()?;

    let dataset_selection = FuzzySelect::new()
        .with_prompt("Which dataset do you want to run this report for?")
        .items(
            &datasets
                .iter()
                .map(|dataset| dataset.full_name().0)
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let dataset = &datasets[dataset_selection];

    info!("Getting dataset summary...");
    let summary_response = client.dataset_summary(&dataset.full_name(), &Default::default())?;

    info!("Getting labellers...");
    let labellers = client.get_labellers(&dataset.full_name())?;

    if labellers.is_empty() {
        bail!("Cannot get a label trend report for a dataset without any pinned models")
    }

    let label_defs = dataset.label_defs.iter();

    let label_inclusion_selections = MultiSelect::new()
        .with_prompt("Select which label(s) you want to include in your report")
        .items(
            &label_defs
                .map(|label| label.name.0.clone())
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let excludable_label_defs: Vec<LabelDef> = dataset
        .label_defs
        .iter()
        .enumerate()
        .filter_map(|(idx, label_def)| {
            if label_inclusion_selections.contains(&idx) {
                None
            } else {
                Some(label_def.clone())
            }
        })
        .collect();

    let label_exclusion_selections = MultiSelect::new()
        .with_prompt("Select which label(s) you want to filter out of your report")
        .items(
            &dataset
                .label_defs
                .iter()
                .enumerate()
                .filter_map(|(idx, label)| {
                    if label_inclusion_selections.contains(&idx) {
                        None
                    } else {
                        Some(label.name.0.clone())
                    }
                })
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let get_threshold_for_selections =
        |selections: Vec<usize>, label_defs: Vec<LabelDef>| -> Vec<(LabelDef, NotNan<f64>)> {
            selections
                .iter()
                .map(|selection| {
                    let label_def = &label_defs[*selection];

                    let confidence_str = Input::new()
                        .with_prompt(format!(
                            "What confidence threshold do you want to use for the label \"{}\"",
                            label_def.name.0
                        ))
                        .validate_with(|input: &String| match input.trim().parse::<NotNan<f64>>() {
                            Ok(number) => {
                                if number >= NotNan::new(0.0).unwrap()
                                    && number <= NotNan::new(1.0).unwrap()
                                {
                                    Ok(())
                                } else {
                                    Err("Please enter a number between 0 and 1")
                                }
                            }

                            Err(_) => Err("Please enter a number between 0 and 1"),
                        })
                        .interact()
                        .expect("Could not get confidence string from user");

                    let confidence = confidence_str
                        .trim()
                        .parse::<NotNan<f64>>()
                        .expect("Could not parse user input");
                    (label_def.clone(), confidence)
                })
                .collect()
        };

    let label_inclusion_by_confidence =
        get_threshold_for_selections(label_inclusion_selections, dataset.label_defs.clone());
    let label_exclusion_by_confidence =
        get_threshold_for_selections(label_exclusion_selections, excludable_label_defs);

    let thresholds = label_inclusion_by_confidence
        .iter()
        .chain(&label_exclusion_by_confidence)
        .map(|(label_def, threshold)| TriggerLabelThreshold {
            threshold: *threshold,
            name: label_def
                .name
                .0
                .split(" > ")
                .map(|part| part.to_string())
                .collect(),
        })
        .collect();

    let model_version_selections = MultiSelect::new()
        .with_prompt("Select which model version(s) you want to run this report for")
        .items(
            &labellers
                .iter()
                .map(|labeller| labeller.version.to_string())
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let model_versions = labellers.iter().enumerate().filter_map(|(idx, labeller)| {
        if model_version_selections.contains(&idx) {
            Some(labeller)
        } else {
            None
        }
    });

    let user_property_filters =
        get_user_properties_filter_interactively(&summary_response.summary)?;

    let get_timestamp = |prompt: String| -> Result<DateTime<Utc>> {
        let date = Input::new()
            .with_prompt(prompt)
            .validate_with(
                |date: &String| match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Please enter the date in the format YYYY-MM-DD"),
                },
            )
            .interact()?;

        Ok(NaiveDate::parse_from_str(&date, "%Y-%m-%d")?
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap())
    };

    let start_timestamp =
        get_timestamp("What date do you want to start your report from (YYYY-MM-DD)".to_string())?;
    let end_timestamp =
        get_timestamp("What date do you want to end your report from (YYYY-MM-DD)".to_string())?;

    let statistics = Arc::new(Statistics::new());

    let make_progress = || -> Result<Progress> {
        let comment_filter = CommentFilter {
            timestamp: Some(CommentTimestampFilter {
                minimum: Some(start_timestamp),
                maximum: Some(end_timestamp),
            }),
            user_properties: Some(user_property_filters.clone()),
            ..Default::default()
        };

        Ok(get_progress_bar(
            *client
                .get_dataset_statistics(
                    &dataset.full_name(),
                    &StatisticsRequestParams {
                        comment_filter,
                        ..Default::default()
                    },
                )
                .context("Operation to get dataset comment count has failed..")?
                .num_comments as u64,
            &statistics,
        ))
    };

    let _progress = make_progress();

    get_label_trend_report(
        client,
        dataset.full_name(),
        &statistics,
        model_versions
            .map(|model| model.version)
            .collect::<Vec<u32>>(),
        user_property_filters,
        thresholds,
        start_timestamp,
        end_timestamp,
    )?;

    todo!()
}

#[allow(clippy::too_many_arguments)]
fn get_label_trend_report(
    client: &Client,
    dataset_name: reinfer_client::DatasetFullName,
    statistics: &Arc<Statistics>,
    model_versions: Vec<u32>,
    user_property_filter: UserPropertiesFilter,
    thresholds: Vec<TriggerLabelThreshold>,
    start_timestamp: DateTime<Utc>,
    end_timestamp: DateTime<Utc>,
) -> Result<()> {
    let mut params = QueryRequestParams {
        continuation: None,
        filter: CommentFilter {
            timestamp: Some(CommentTimestampFilter {
                minimum: Some(start_timestamp),
                maximum: Some(end_timestamp),
            }),
            user_properties: Some(user_property_filter),
            ..Default::default()
        },
        limit: DEFAULT_QUERY_PAGE_SIZE,
        order: OrderEnum::Recent,
        ..Default::default()
    };

    client
        .get_dataset_query_iter(&dataset_name, &mut params)
        .try_for_each(|page| {
            let page = page.context("Operation to get comments has failed.")?;
            if page.is_empty() {
                return Ok(());
            }

            statistics.add_comments(page.len());

            for model_version in &model_versions {
                let predictions: Vec<Prediction> = client
                    .get_comment_predictions(
                        &dataset_name,
                        &ModelVersion(*model_version),
                        page.iter().map(|comment| &comment.comment.uid),
                        None,
                        Some(thresholds.clone()),
                    )
                    .context("Operation to get predictions has failed.")?;

                statistics.add_predicted(page.len())
            }
            Ok(())
        })
}

#[derive(Debug, Default)]
pub struct Statistics {
    downloaded: AtomicUsize,
    predicted: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            downloaded: AtomicUsize::new(0),
            predicted: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_comments(&self, num_downloaded: usize) {
        self.downloaded.fetch_add(num_downloaded, Ordering::SeqCst);
    }

    #[inline]
    fn add_predicted(&self, num_predicted: usize) {
        self.predicted.fetch_add(num_predicted, Ordering::SeqCst);
    }

    #[inline]
    fn num_downloaded(&self) -> usize {
        self.downloaded.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_predicted(&self) -> usize {
        self.predicted.load(Ordering::SeqCst)
    }
}

fn get_progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let num_downloaded = statistics.num_downloaded();
            let num_predicted = statistics.num_predicted();
            (
                num_downloaded as u64,
                format!(
                    "{} {} {} {}",
                    num_downloaded.to_string().bold(),
                    "comments".dimmed(),
                    num_predicted.to_string().bold(),
                    "predictions".dimmed()
                ),
            )
        },
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: false },
    )
}
