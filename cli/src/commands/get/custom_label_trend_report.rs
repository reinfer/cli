use core::f64;
use std::{
    collections::HashMap,
    fs, mem,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use colored::Colorize;
use csv::Writer;
use dialoguer::{FuzzySelect, Input, MultiSelect};
use log::{info, warn};
use ordered_float::NotNan;
use reinfer_client::{
    resources::{
        comment::{CommentTimestampFilter, UserPropertiesFilter},
        dataset::{
            OrderEnum, QueryRequestParams, StatisticsRequestParams, SummaryResponse,
            UserModelMetadata,
        },
    },
    AnnotatedComment, Client, CommentFilter, Dataset, Entities, LabelDef, LabelName, Labelling,
    ModelVersion, PredictedLabel, Prediction, TriggerLabelThreshold, DEFAULT_LABEL_GROUP_NAME,
};
use structopt::StructOpt;

use crate::{
    commands::get::comments::get_user_properties_filter_interactively,
    printer::Printer,
    progress::{Options as ProgressOptions, Progress},
};

use super::comments::DEFAULT_QUERY_PAGE_SIZE;

const PATH_PRINT_SEPERATOR: &str = "\n - ";
const OUTPUT_FOLDER_PREFIX: &str = "LabelTrendReport";
const MAX_COMMENT_SAMPLE: u64 = 150000;

#[derive(Debug, StructOpt)]
pub struct GetCustomLabelTrendReportArgs {}

struct LabelTrendReportParams {
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
    pub labels_to_include: Vec<LabelName>,
    pub labels_to_exclude: Vec<LabelName>,
    pub user_property_filters: UserPropertiesFilter,
    pub thresholds: HashMap<LabelName, TriggerLabelThreshold>,
}

impl LabelTrendReportParams {
    pub fn get_query_request_params(&self, limit: Option<usize>) -> QueryRequestParams {
        QueryRequestParams {
            continuation: None,
            filter: get_comment_filter(
                self.start_timestamp,
                self.end_timestamp,
                self.user_property_filters.clone(),
            ),
            limit,
            order: OrderEnum::Sample { seed: 42 },
            ..Default::default()
        }
    }
}

pub fn get_dataset_selection(client: &Client) -> Result<Dataset> {
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

    Ok(datasets[dataset_selection].clone())
}

pub fn pop_label_selection(prompt: &str, label_defs: &mut Vec<LabelDef>) -> Result<Vec<LabelName>> {
    let label_inclusion_selections = MultiSelect::new()
        .with_prompt(prompt)
        .items(
            &label_defs
                .iter()
                .map(|label| label.name.0.clone())
                .collect::<Vec<String>>(),
        )
        .interact()?;

    Ok(label_inclusion_selections
        .iter()
        .rev()
        .map(|selection| label_defs.remove(*selection).name)
        .collect())
}

fn get_dataset_info(
    client: &Client,
    dataset: &Dataset,
) -> Result<(SummaryResponse, Vec<UserModelMetadata>)> {
    info!("Getting dataset summary...");
    let summary_response = client.dataset_summary(&dataset.full_name(), &Default::default())?;

    info!("Getting labellers...");
    let labellers = client.get_labellers(&dataset.full_name())?;

    if labellers.is_empty() {
        bail!("Cannot get a label trend report for a dataset without any pinned models")
    }
    Ok((summary_response, labellers))
}

fn get_threshold_for_label_selections(
    label_defs: Vec<&LabelName>,
) -> HashMap<LabelName, TriggerLabelThreshold> {
    label_defs
        .into_iter()
        .map(|label_name| {
            let confidence_str = Input::new()
                .with_prompt(format!(
                    "What confidence threshold do you want to use for the label \"{}\"",
                    label_name.0
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

            let threshold: NotNan<f64> = confidence_str
                .trim()
                .parse::<NotNan<f64>>()
                .expect("Could not parse user input");

            (
                label_name.clone(),
                TriggerLabelThreshold {
                    threshold,
                    name: label_name
                        .0
                        .split(" > ")
                        .map(|part| part.to_string())
                        .collect(),
                },
            )
        })
        .collect()
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum ModelVersionSelection {
    ModelVersion(ModelVersion),
    Latest,
}

fn get_model_version_selection(
    labellers: &[UserModelMetadata],
) -> Result<Vec<ModelVersionSelection>> {
    let labellers_as_string: Vec<String> = labellers
        .iter()
        .map(|labeller| labeller.version.0.to_string())
        .collect::<Vec<String>>();
    let mut options = vec!["Latest".to_string()];
    options.extend(labellers_as_string);

    let model_version_selections = MultiSelect::new()
        .with_prompt("Select which model version(s) you want to run this report for")
        .items(&options)
        .interact()?;

    Ok(model_version_selections
        .iter()
        .map(|selection| {
            if *selection == 0 {
                ModelVersionSelection::Latest
            } else {
                let labeller = labellers[*selection].clone();
                ModelVersionSelection::ModelVersion(labeller.version)
            }
        })
        .collect())
}

fn get_timestamp(prompt: &str) -> Result<DateTime<Utc>> {
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
}

fn get_comment_filter(
    start_timestamp: DateTime<Utc>,
    end_timestamp: DateTime<Utc>,
    user_property_filters: UserPropertiesFilter,
) -> CommentFilter {
    CommentFilter {
        timestamp: Some(CommentTimestampFilter {
            minimum: Some(start_timestamp),
            maximum: Some(end_timestamp),
        }),
        user_properties: Some(user_property_filters.clone()),
        ..Default::default()
    }
}

pub fn get_comment_count(
    client: &Client,
    dataset: &Dataset,
    comment_filter: CommentFilter,
) -> Result<u64> {
    let num_comments: f64 = client
        .get_dataset_statistics(
            &dataset.full_name(),
            &StatisticsRequestParams {
                comment_filter: comment_filter.clone(),
                ..Default::default()
            },
        )
        .context("Operation to get dataset comment count has failed..")?
        .num_comments
        .into();

    Ok(num_comments as u64)
}

pub fn get(
    client: &Client,
    _args: &GetCustomLabelTrendReportArgs,
    _printer: &Printer,
) -> Result<()> {
    let dataset = get_dataset_selection(client)?;

    let (summary_response, labellers) = get_dataset_info(client, &dataset)?;

    let mut label_defs = dataset.label_defs.clone();

    let labels_to_include = pop_label_selection(
        "Select which label(s) you want to include in your report",
        &mut label_defs,
    )?;

    let labels_to_exclude = pop_label_selection(
        "Select which label(s) you want to filter out of your report",
        &mut label_defs,
    )?;

    let thresholds = get_threshold_for_label_selections(
        labels_to_include
            .iter()
            .chain(labels_to_exclude.iter())
            .collect(),
    );

    let model_versions = get_model_version_selection(&labellers)?;

    let user_property_filters =
        get_user_properties_filter_interactively(&summary_response.summary)?;

    let start_timestamp =
        get_timestamp("What date do you want to start your report from (YYYY-MM-DD)")?;
    let end_timestamp =
        get_timestamp("What date do you want to end your report from (YYYY-MM-DD)")?;

    let statistics = Arc::new(Statistics::new());

    let comment_filter = get_comment_filter(
        start_timestamp,
        end_timestamp,
        user_property_filters.clone(),
    );

    let total_comment_count = get_comment_count(client, &dataset, comment_filter)?;

    let report_multiply_ratio = if total_comment_count >= MAX_COMMENT_SAMPLE {
        warn!(
            "Dataset size too big, sampling from first {MAX_COMMENT_SAMPLE}"
        );
        total_comment_count / MAX_COMMENT_SAMPLE
    } else {
        1
    };

    let target_comment_count = if total_comment_count <= MAX_COMMENT_SAMPLE {
        total_comment_count
    } else {
        MAX_COMMENT_SAMPLE
    };

    let _progress = get_progress_bar(target_comment_count, &statistics);

    let label_trend_report_params = LabelTrendReportParams {
        start_timestamp,
        end_timestamp,
        labels_to_include,
        labels_to_exclude,
        user_property_filters,
        thresholds,
    };

    let mut report = get_label_trend_report(
        client,
        &dataset.full_name(),
        &statistics,
        model_versions,
        &label_trend_report_params,
        target_comment_count as usize,
    )?;

    report.scale_sampled_results(report_multiply_ratio as usize);

    let csv_paths = report.write_csvs_to_desktop()?;
    info!(
        "Saved CSV(s) to desktop:{PATH_PRINT_SEPERATOR}{}",
        csv_paths
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(PATH_PRINT_SEPERATOR)
    );
    Ok(())
}

fn get_label_trend_report_from_json(
    client: &Client,
    dataset_name: &reinfer_client::DatasetFullName,
    statistics: &Arc<Statistics>,
    model_version: &ModelVersion,
    params: &LabelTrendReportParams,
    report: &mut Report,
) -> Result<()> {
    let LabelTrendReportParams {
        labels_to_include,
        labels_to_exclude,
        thresholds,
        ..
    } = params;

    let mut query_params = params.get_query_request_params(Some(DEFAULT_QUERY_PAGE_SIZE));
    let query_iter = client.get_dataset_query_iter(dataset_name, &mut query_params);

    for page in query_iter.take(MAX_COMMENT_SAMPLE as usize) {
        let page = page.context("Operation to get comments has failed.")?;
        if page.is_empty() {
            return Ok(());
        }
        statistics.add_comments(page.len());

        let predictions: Vec<Prediction> = client
            .get_comment_predictions(
                dataset_name,
                model_version,
                page.iter().map(|comment| &comment.comment.uid),
                None,
                Some(thresholds.clone().into_values().collect()),
            )
            .context("Operation to get predictions has failed.")?;

        let comments: Vec<_> = page
            .clone()
            .into_iter()
            .zip(predictions)
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
                                auto_thresholds: None,
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

        for comment in comments {
            let predicted_labels = comment
                .labelling
                .context("Could not get labelling for comment")?[0]
                .predicted
                .clone()
                .context("Could not get predicted labels for comment")?
                .iter()
                .map(|predicted| predicted.name.to_label_name())
                .collect();

            count_for_predicted_labels(
                &predicted_labels,
                labels_to_include,
                labels_to_exclude,
                &ModelVersionSelection::ModelVersion(model_version.clone()),
                comment.comment.timestamp.date_naive(),
                report,
            )
        }

        statistics.add_predicted(page.len());
    }
    Ok(())
}

fn count_for_predicted_labels(
    predicted_labels: &Vec<LabelName>,
    labels_to_include: &[LabelName],
    labels_to_exclude: &[LabelName],
    model_version: &ModelVersionSelection,
    timestamp: NaiveDate,
    report: &mut Report,
) {
    if predicted_labels.iter().any(|label| {
        let name = label;
        labels_to_exclude.contains(name)
    }) {
        return;
    }

    for label in predicted_labels {
        if labels_to_include.contains(label) {
            report.count_label(model_version, timestamp, label);
            // only count each message once
            return;
        }
    }
}

fn get_label_trend_report(
    client: &Client,
    dataset_name: &reinfer_client::DatasetFullName,
    statistics: &Arc<Statistics>,
    model_version_selections: Vec<ModelVersionSelection>,
    params: &LabelTrendReportParams,
    target_comment_count: usize,
) -> Result<Report> {
    let LabelTrendReportParams {
        start_timestamp,
        end_timestamp,
        ..
    } = params;

    let mut report: Report = Report::new(start_timestamp.date_naive(), end_timestamp.date_naive());

    for model_version_selection in &model_version_selections {
        match model_version_selection {
            ModelVersionSelection::Latest => get_label_trend_report_from_csv(
                client,
                dataset_name,
                statistics,
                params,
                &mut report,
                target_comment_count,
            )?,
            ModelVersionSelection::ModelVersion(model_version) => get_label_trend_report_from_json(
                client,
                dataset_name,
                statistics,
                model_version,
                params,
                &mut report,
            )?,
        }
    }

    Ok(report)
}

fn has_exceeded_threshold(
    record: &HashMap<String, String>,
    label: &LabelName,
    threshold: &TriggerLabelThreshold,
) -> bool {
    let value_for_label_str: String = record[&label.0].clone();
    let value_for_label: f64 = value_for_label_str
        .parse()
        .unwrap_or_else(|_| panic!("Could not parse value '{value_for_label_str}' as f64"));
    value_for_label > *threshold.threshold
}

fn get_label_trend_report_from_csv(
    client: &Client,
    dataset_name: &reinfer_client::DatasetFullName,
    statistics: &Arc<Statistics>,
    params: &LabelTrendReportParams,
    report: &mut Report,
    limit: usize,
) -> Result<()> {
    let LabelTrendReportParams {
        labels_to_include,
        labels_to_exclude,
        thresholds,
        ..
    } = params;

    let params = params.get_query_request_params(Some(limit));

    let csv_string = client.query_dataset_csv(dataset_name, &params)?;

    let mut rdr = csv::Reader::from_reader(csv_string.as_bytes());

    for result in rdr.deserialize() {
        let record: HashMap<String, String> = result?;

        let mut predicted_labels: Vec<LabelName> = Vec::new();
        for (label, threshold) in thresholds {
            if has_exceeded_threshold(&record, label, threshold) {
                predicted_labels.push(label.clone())
            }
        }

        let timestamp_str = &record["Date"];
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?;

        count_for_predicted_labels(
            &predicted_labels,
            labels_to_include,
            labels_to_exclude,
            &ModelVersionSelection::Latest,
            timestamp.date_naive(),
            report,
        );

        statistics.add_comments(1);
        statistics.add_predicted(1);
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct Statistics {
    downloaded: AtomicUsize,
    predicted: AtomicUsize,
}

#[derive(Default)]
pub struct DateEntry(HashMap<LabelName, usize>);

impl DateEntry {
    fn count_label(&mut self, label: &LabelName) {
        *self.0.entry(label.clone()).or_default() += 1;
    }

    fn get_label_count(&self, label: &LabelName) -> usize {
        *self.0.get(label).unwrap_or(&0_usize)
    }
}

#[derive(Default)]
pub struct ModelVersionEntry(HashMap<NaiveDate, DateEntry>);

impl ModelVersionEntry {
    fn get_date_entry_mut(&mut self, date: NaiveDate) -> &mut DateEntry {
        self.0.entry(date).or_default()
    }
}

#[derive(Default)]
pub struct Report {
    volume_by_date: HashMap<ModelVersionSelection, ModelVersionEntry>,
    labels: Vec<LabelName>,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl Report {
    pub fn scale_sampled_results(&mut self, scale: usize) {
        self.volume_by_date.values_mut().for_each(|model_entry| {
            model_entry.0.values_mut().for_each(|value_entry| {
                value_entry.0.values_mut().for_each(|value| *value *= scale)
            })
        });
    }

    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            start_date,
            end_date,
            ..Default::default()
        }
    }

    pub fn write_csvs_to_desktop(&mut self) -> Result<Vec<PathBuf>> {
        let desktop_dir = dirs::desktop_dir().context("Could not get user's desktop path")?;
        let timestamp = chrono::offset::Local::now()
            .format("%Y-%m-%d-%H-%M-%S")
            .to_string();
        let folder_name = format!("{OUTPUT_FOLDER_PREFIX}-{timestamp}");

        let output_dir = desktop_dir.join(folder_name);
        fs::create_dir(&output_dir)?;
        self.write_csvs(&output_dir)
    }

    pub fn write_csvs(&mut self, output_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut paths: Vec<PathBuf> = Vec::new();
        for (model_version, model_version_entry) in self.volume_by_date.iter_mut() {
            let path = output_dir.join(format!(
                "{}.csv",
                match model_version {
                    ModelVersionSelection::Latest => "Latest".to_string(),
                    ModelVersionSelection::ModelVersion(verson) => verson.0.to_string(),
                }
            ));
            paths.push(path.clone());

            let mut wtr = Writer::from_path(&path)?;

            let date_range = DateRange(self.start_date, self.end_date);

            wtr.write_record(
                vec!["date".to_string()]
                    .into_iter()
                    .chain(self.labels.iter().map(|label| label.0.clone())),
            )?;

            for date in date_range {
                let date_entry = model_version_entry.get_date_entry_mut(date);
                let record = vec![date.to_string()].into_iter().chain(
                    self.labels
                        .iter()
                        .map(|label| date_entry.get_label_count(label).to_string()),
                );
                wtr.write_record(record)?;
            }
        }

        Ok(paths)
    }

    fn count_label(
        &mut self,
        model_version: &ModelVersionSelection,
        date: NaiveDate,
        label: &LabelName,
    ) {
        if !self.labels.contains(label) {
            self.labels.push(label.clone())
        }

        self.get_model_version_entry_mut(model_version)
            .get_date_entry_mut(date)
            .count_label(label);
    }

    fn get_model_version_entry_mut(
        &mut self,
        model_version: &ModelVersionSelection,
    ) -> &mut ModelVersionEntry {
        self.volume_by_date
            .entry(model_version.clone())
            .or_default()
    }
}

struct DateRange(NaiveDate, NaiveDate);

impl Iterator for DateRange {
    type Item = NaiveDate;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 <= self.1 {
            let next = self.0 + Duration::days(1);
            Some(mem::replace(&mut self.0, next))
        } else {
            None
        }
    }
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
