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
use structopt::StructOpt;

use openapi::{
    apis::{
        configuration::Configuration,
        datasets_api::{get_all_datasets, get_dataset_statistics, get_dataset_summary},
        models_api::{get_all_models_in_dataset, get_comment_predictions},
    },
    models::{
        self, _query_comments_order_sample::Kind, labelling_group::Group, AnnotatedComment,
        CommentFilter, Dataset, GetAllModelsInDatasetRequest, GetCommentPredictionsRequest,
        GetDatasetStatisticsRequest, LabelDef, LabellingGroup, Order, QueryCommentsOrderSample,
        QueryCommentsRequest, TimestampRangeFilter, TriggerLabelThreshold,
    },
};

use crate::{
    commands::get::comments::{get_user_properties_filter_interactively, DEFAULT_QUERY_PAGE_SIZE},
    printer::Printer,
    progress::{Options as ProgressOptions, Progress},
    utils::{csv::query_comments_csv, get_dataset_query_iter, ModelVersion, UserPropertiesFilter},
};

const PATH_PRINT_SEPERATOR: &str = "\n - ";
const OUTPUT_FOLDER_PREFIX: &str = "LabelTrendReport";
const MAX_COMMENT_SAMPLE: u64 = 150000;

#[derive(Debug, StructOpt)]
pub struct GetCustomLabelTrendReportArgs {}

struct LabelTrendReportParams {
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: DateTime<Utc>,
    pub labels_to_include: Vec<String>,
    pub labels_to_exclude: Vec<String>,
    pub user_property_filters: UserPropertiesFilter,
    pub thresholds: HashMap<String, TriggerLabelThreshold>,
}

/// Get user selection for dataset from available datasets
pub fn get_dataset_selection(config: &Configuration) -> Result<Box<Dataset>> {
    info!("Getting datasets...");
    let datasets = get_sorted_datasets(config)?;

    let dataset_names: Vec<String> = datasets
        .iter()
        .map(|dataset| format!("{}/{}", dataset.owner, dataset.name))
        .collect();

    let dataset_selection = FuzzySelect::new()
        .with_prompt("Which dataset do you want to run this report for?")
        .items(&dataset_names)
        .interact()
        .context("Failed to get user input for dataset selection")?;

    Ok(Box::new(datasets[dataset_selection].clone()))
}

/// Get all datasets sorted by owner and name
fn get_sorted_datasets(config: &Configuration) -> Result<Vec<Dataset>> {
    let datasets_response = get_all_datasets(config).context("Failed to get datasets")?;

    let mut datasets = datasets_response.datasets;
    datasets.sort_unstable_by(|a, b| (&a.owner, &a.name).cmp(&(&b.owner, &b.name)));

    Ok(datasets)
}

/// Interactive selection of labels from available label definitions
pub fn pop_label_selection(prompt: &str, label_defs: &mut Vec<LabelDef>) -> Result<Vec<String>> {
    let label_inclusion_selections = MultiSelect::new()
        .with_prompt(prompt)
        .items(
            &label_defs
                .iter()
                .map(|label| label.name.clone())
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
    config: &Configuration,
    dataset: &Dataset,
) -> Result<(
    models::GetDatasetSummaryResponse,
    Vec<models::UserModelMetadata>,
)> {
    info!("Getting dataset summary...");
    let summary_response = get_dataset_summary(
        config,
        &dataset.owner,
        &dataset.name,
        models::GetDatasetSummaryRequest::default(),
    )?;

    info!("Getting labellers...");
    let models_request = GetAllModelsInDatasetRequest::new();
    let models_response =
        get_all_models_in_dataset(config, &dataset.owner, &dataset.name, models_request)?;
    let labellers = models_response.labellers;

    if labellers.is_empty() {
        bail!("Cannot get a label trend report for a dataset without any pinned models")
    }
    Ok((summary_response, labellers))
}

fn get_threshold_for_label_selections(
    label_defs: Vec<&String>,
) -> HashMap<String, TriggerLabelThreshold> {
    label_defs
        .into_iter()
        .map(|label_name| {
            let confidence_str = Input::new()
                .with_prompt(format!(
                    "What confidence threshold do you want to use for the label \"{label_name}\""
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
                    threshold: Some(threshold.into_inner()),
                    name: label_name
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
    labellers: &[models::UserModelMetadata],
) -> Result<Vec<ModelVersionSelection>> {
    let labellers_as_string: Vec<String> = labellers
        .iter()
        .map(|labeller| labeller.version.to_string())
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
                let labeller = labellers[*selection - 1].clone();
                ModelVersionSelection::ModelVersion(ModelVersion(labeller.version))
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
        timestamp: Some(Box::new(TimestampRangeFilter {
            minimum: Some(start_timestamp.to_rfc3339()),
            maximum: Some(end_timestamp.to_rfc3339()),
        })),
        user_properties: Some(serde_json::to_value(user_property_filters).unwrap()),
        ..Default::default()
    }
}

/// Get the total count of comments in a dataset with filtering
pub fn get_comment_count(
    config: &Configuration,
    dataset: &Dataset,
    comment_filter: CommentFilter,
) -> Result<u64> {
    let stats_response = get_dataset_statistics(
        config,
        &dataset.owner,
        &dataset.name,
        GetDatasetStatisticsRequest {
            comment_filter: Some(comment_filter),
            ..Default::default()
        },
    )
    .context("Operation to get dataset comment count has failed..")?;

    Ok(stats_response.statistics.num_comments as u64)
}

/// Generate an interactive custom label trend report
pub fn get(
    config: &Configuration,
    _args: &GetCustomLabelTrendReportArgs,
    _printer: &Printer,
) -> Result<()> {
    let dataset = get_dataset_selection(config)?;

    let (summary_response, labellers) = get_dataset_info(config, &dataset)?;

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

    let total_comment_count = get_comment_count(config, &dataset, comment_filter)?;

    let report_multiply_ratio = if total_comment_count >= MAX_COMMENT_SAMPLE {
        warn!("Dataset size too big, sampling from first {MAX_COMMENT_SAMPLE}");
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
        config,
        &dataset.owner,
        &dataset.name,
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
    config: &Configuration,
    dataset_owner: &str,
    dataset_name: &str,
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

    let request = QueryCommentsRequest {
        continuation: None,
        limit: Some(DEFAULT_QUERY_PAGE_SIZE as i32),
        attribute_filters: None,
        collapse_mode: None,
        filter: Some(get_comment_filter(
            params.start_timestamp,
            params.end_timestamp,
            params.user_property_filters.clone(),
        )),
        order: Box::new(Order::Sample(Box::new(QueryCommentsOrderSample {
            kind: Kind::Sample,
            seed: 42,
        }))),
    };

    let query_iter = get_dataset_query_iter(
        config.clone(),
        dataset_owner.to_string(),
        dataset_name.to_string(),
        request,
    );

    for page_result in query_iter.take(MAX_COMMENT_SAMPLE as usize) {
        let page = page_result.context("Operation to get comments has failed.")?;
        if page.is_empty() {
            return Ok(());
        }
        statistics.add_comments(page.len());

        let uids: Vec<String> = page
            .iter()
            .map(|comment| comment.comment.uid.clone())
            .collect();

        // Convert thresholds HashMap to Vec<TriggerLabelThreshold> for API request
        let threshold_labels: Vec<TriggerLabelThreshold> = thresholds.values().cloned().collect();

        let prediction_request = GetCommentPredictionsRequest {
            uids,
            threshold: None, // Could set a global threshold here if needed
            labels: if threshold_labels.is_empty() {
                None
            } else {
                Some(threshold_labels)
            },
        };

        let predictions_response = get_comment_predictions(
            config,
            dataset_owner,
            dataset_name,
            &model_version.to_string(),
            prediction_request,
        )
        .context("Operation to get predictions has failed.")?;

        let predictions = predictions_response.predictions;

        let comments: Vec<_> = page
            .clone()
            .into_iter()
            .zip(predictions)
            .map(|(comment, prediction)| AnnotatedComment {
                comment: comment.comment,
                labelling: vec![LabellingGroup {
                    group: Group::Default,
                    uninformative: None,
                    assigned: Vec::new(),
                    dismissed: Vec::new(),
                    predicted: Some(prediction.labels),
                }],
                reviewable_blocks: None,
                entities: prediction.entities.map(|entities| {
                    Box::new(models::Entities {
                        assigned: Vec::new(),
                        dismissed: Vec::new(),
                        predicted: entities,
                    })
                }),
                thread_properties: None,
                trigger_exceptions: None,
                label_properties: None,
                moon_forms: None,
                extractions: None,
                highlights: None,
                diagnostics: None,
                model_version: None,
            })
            .collect();

        for comment in comments {
            if comment.labelling.is_empty() {
                continue;
            }
            let predicted_labels = if let Some(predicted) = &comment.labelling[0].predicted {
                predicted
                    .iter()
                    .map(|predicted| {
                        crate::utils::conversions::convert_boxed_name_to_label_name(&predicted.name)
                    })
                    .collect()
            } else {
                continue;
            };

            count_for_predicted_labels(
                &predicted_labels,
                labels_to_include,
                labels_to_exclude,
                &ModelVersionSelection::ModelVersion(model_version.clone()),
                comment
                    .comment
                    .timestamp
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap()
                    .date_naive(),
                report,
            )
        }

        statistics.add_predicted(page.len());
    }
    Ok(())
}

fn count_for_predicted_labels(
    predicted_labels: &Vec<String>,
    labels_to_include: &[String],
    labels_to_exclude: &[String],
    model_version: &ModelVersionSelection,
    timestamp: NaiveDate,
    report: &mut Report,
) {
    if predicted_labels
        .iter()
        .any(|label| labels_to_exclude.contains(label))
    {
        return;
    }

    for label in predicted_labels {
        if labels_to_include.contains(label) {
            report.count_label(model_version, timestamp, label);
            return;
        }
    }
}

fn get_label_trend_report(
    config: &Configuration,
    owner: &str,
    dataset_name: &str,
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
                config,
                owner,
                dataset_name,
                statistics,
                params,
                &mut report,
                target_comment_count,
            )?,
            ModelVersionSelection::ModelVersion(model_version) => get_label_trend_report_from_json(
                config,
                owner,
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
    label: &String,
    threshold: &TriggerLabelThreshold,
) -> bool {
    let value_for_label_str: String = record[label].clone();
    let value_for_label: f64 = value_for_label_str
        .parse()
        .unwrap_or_else(|_| panic!("Could not parse value '{value_for_label_str}' as f64"));
    value_for_label > threshold.threshold.unwrap_or(0.5)
}

fn get_label_trend_report_from_csv(
    config: &Configuration,
    owner: &str,
    dataset_name: &str,
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

    let query_request = QueryCommentsRequest {
        filter: Some(get_comment_filter(
            params.start_timestamp,
            params.end_timestamp,
            params.user_property_filters.clone(),
        )),
        order: Box::new(Order::Sample(Box::new(QueryCommentsOrderSample::new(
            Kind::Sample,
            42,
        )))),
        ..Default::default()
    };

    let csv_string = query_comments_csv(
        config,
        owner,
        dataset_name,
        query_request,
        Some(limit as i32),
        None,
        None,
        None,
    )?;

    let mut rdr = csv::Reader::from_reader(csv_string.as_bytes());

    for result in rdr.deserialize() {
        let record: HashMap<String, String> = result?;

        let mut predicted_labels: Vec<String> = Vec::new();
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
pub struct DateEntry(HashMap<String, usize>);

impl DateEntry {
    fn count_label(&mut self, label: &str) {
        *self.0.entry(label.to_owned()).or_default() += 1;
    }

    fn get_label_count(&self, label: &String) -> usize {
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
    labels: Vec<String>,
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
                    .chain(self.labels.iter().cloned()),
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
        label: &String,
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
