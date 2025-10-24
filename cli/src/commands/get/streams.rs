use std::{
    fs::File,
    io::{self, BufWriter},
    path::PathBuf,
    sync::mpsc::channel,
};

use anyhow::{anyhow, Context, Result};
use colored::{ColoredString, Colorize};
use log::info;
use ordered_float::NotNan;
use prettytable::row;
use scoped_threadpool::Pool;
use serde::Serialize;
use structopt::StructOpt;

use openapi::{
    apis::{
        configuration::Configuration,
        models_api::{get_label_validation, get_validation},
        streams_api::{advance_stream, fetch_from_stream, get_all_streams, get_stream_by_name},
    },
    models::{
        AdvanceStreamRequest, FetchFromStreamRequest, GetLabelValidationRequest,
        GetValidationResponse, LabelDef, LabelMetrics, Trigger, TriggerLabelThreshold,
        TriggerUserModel,
    },
};

use crate::{
    printer::{print_resources_as_json, DisplayTable, Printer},
    utils::{resolve_dataset, DatasetFullName, DatasetIdentifier, ModelVersion, StreamFullName},
};

#[derive(Debug, StructOpt)]
pub struct GetStreamsArgs {
    #[structopt(short = "d", long = "dataset")]
    /// The dataset name or id
    dataset: DatasetIdentifier,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write streams as JSON.
    path: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct GetStreamCommentsArgs {
    #[structopt(long = "stream")]
    /// The full stream name `<owner>/<dataset>/<stream>`.
    stream: StreamFullName,

    #[structopt(long = "size", default_value = "16")]
    /// The max number of comments to return per batch.
    size: u32,

    #[structopt(long = "listen")]
    /// If set, the command will run forever polling every N seconds and advancing the stream.
    listen: Option<f64>,

    #[structopt(long = "individual-advance")]
    /// If set, the command will acknowledge each comment in turn, rather than full batches.
    individual_advance: bool,
}

#[derive(Debug, StructOpt)]
pub struct GetStreamStatsArgs {
    #[structopt(name = "stream")]
    /// The full stream name `<owner>/<dataset>/<stream>`.
    stream_full_name: StreamFullName,

    #[structopt(long = "compare-version", short = "v")]
    /// The model version to compare stats with
    compare_to_model_version: Option<ModelVersion>,

    #[structopt(long = "compare-dataset", short = "d")]
    /// The dataset to compare stats with
    compare_to_dataset: Option<DatasetFullName>,
}

/// Retrieve streams for a dataset with optional file output
pub fn get(config: &Configuration, args: &GetStreamsArgs, printer: &Printer) -> Result<()> {
    let GetStreamsArgs { dataset, path } = args;

    let dataset = resolve_dataset(config, dataset)?;
    let streams = get_streams_sorted(config, &dataset)?;

    output_streams(streams, path, printer)
}

/// Get all streams for a dataset, sorted by name
fn get_streams_sorted(
    config: &Configuration,
    dataset: &openapi::models::Dataset,
) -> Result<Vec<Trigger>> {
    let streams_response =
        get_all_streams(config, &dataset.owner, &dataset.name).with_context(|| {
            format!(
                "Failed to get streams for dataset {}/{}",
                dataset.owner, dataset.name
            )
        })?;

    let mut streams = streams_response.streams;
    streams.sort_unstable_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    Ok(streams)
}

/// Output streams either to file or printer
fn output_streams(streams: Vec<Trigger>, path: &Option<PathBuf>, printer: &Printer) -> Result<()> {
    match path {
        Some(file_path) => {
            let file = File::create(file_path)
                .with_context(|| {
                    format!("Could not open file for writing `{}`", file_path.display())
                })
                .map(BufWriter::new)?;
            print_resources_as_json(streams, Box::new(file))
        }
        None => printer.print_resources(&streams),
    }
}

#[derive(Serialize)]
pub struct StreamStat {
    label_name: String,
    threshold: NotNan<f64>,
    precision: NotNan<f64>,
    recall: NotNan<f64>,
    compare_to_precision: Option<NotNan<f64>>,
    compare_to_recall: Option<NotNan<f64>>,
    maintain_recall_precision: Option<NotNan<f64>>,
    maintain_recall_threshold: Option<NotNan<f64>>,
    maintain_precision_recall: Option<NotNan<f64>>,
    maintain_precision_threshold: Option<NotNan<f64>>,
}
impl DisplayTable for StreamStat {
    fn to_table_headers() -> prettytable::Row {
        row![
            "Name",
            "Threshold (T)",
            "Precision (P)",
            "Recall (R)",
            "P at same T",
            "R at same T",
            "P at same R",
            "R at same P",
            "T at same R",
            "T at same P"
        ]
    }
    fn to_table_row(&self) -> prettytable::Row {
        row![
            self.label_name,
            format!("{:.3}", self.threshold),
            format!("{:.3}", self.precision),
            format!("{:.3}", self.recall),
            if let Some(precision) = self.compare_to_precision {
                red_if_lower_green_otherwise(precision, self.precision)
            } else {
                "none".dimmed()
            },
            if let Some(recall) = self.compare_to_recall {
                red_if_lower_green_otherwise(recall, self.recall)
            } else {
                "none".dimmed()
            },
            if let Some(precision) = self.maintain_recall_precision {
                red_if_lower_green_otherwise(precision, self.precision)
            } else {
                "none".dimmed()
            },
            if let Some(recall) = self.maintain_precision_recall {
                red_if_lower_green_otherwise(recall, self.recall)
            } else {
                "none".dimmed()
            },
            if let Some(threshold) = self.maintain_recall_threshold {
                format!("{threshold:.5}").normal()
            } else {
                "none".dimmed()
            },
            if let Some(threshold) = self.maintain_precision_threshold {
                format!("{threshold:.5}").normal()
            } else {
                "none".dimmed()
            }
        ]
    }
}

fn red_if_lower_green_otherwise(test: NotNan<f64>, threshold: NotNan<f64>) -> ColoredString {
    let test_str = format!("{test:.3}");

    let diff = test - threshold;

    match test {
        test if test < threshold => format!("{test_str} ({diff:+.3})").red(),
        test if test > threshold => format!("{test_str} ({diff:+.3})").green(),
        _ => test_str.green(),
    }
}

#[derive(Default)]
struct ThresholdAndPrecision {
    threshold: Option<NotNan<f64>>,
    precision: Option<NotNan<f64>>,
}

fn get_threshold_and_precision_for_recall(
    recall: NotNan<f64>,
    label_name: &String,
    label_validation: &LabelMetrics,
) -> Result<ThresholdAndPrecision> {
    let recall_index = label_validation
        .recalls
        .iter()
        .position(|&val_recall| val_recall >= *recall)
        .context(format!("Could not get recall for label {}", label_name))?;

    let precision = label_validation.precisions.get(recall_index);

    let threshold = label_validation.thresholds.get(recall_index);

    Ok(ThresholdAndPrecision {
        threshold: threshold.map(|t| NotNan::new(*t).unwrap()),
        precision: precision.map(|p| NotNan::new(*p).unwrap()),
    })
}

#[derive(Default)]
struct ThresholdAndRecall {
    threshold: Option<NotNan<f64>>,
    recall: Option<NotNan<f64>>,
}

fn get_threshold_and_recall_for_precision(
    precision: NotNan<f64>,
    label_name: &String,
    label_validation: &LabelMetrics,
) -> Result<ThresholdAndRecall> {
    // Get lowest index with greater than or equal precision
    let mut precision_index = None;
    label_validation
        .precisions
        .iter()
        .enumerate()
        .for_each(|(idx, val_precision)| {
            if val_precision >= &precision {
                precision_index = Some(idx);
            }
        });

    let precision_index = precision_index.context(format!(
        "Could not get precision index for label {}",
        label_name
    ))?;

    let recall = label_validation.recalls.get(precision_index);
    let threshold = label_validation.thresholds.get(precision_index);

    Ok(ThresholdAndRecall {
        threshold: threshold.map(|t| NotNan::new(*t).unwrap()),
        recall: recall.map(|r| NotNan::new(*r).unwrap()),
    })
}

#[derive(Default)]
struct PrecisionAndRecall {
    precision: NotNan<f64>,
    recall: NotNan<f64>,
}

fn get_precision_and_recall_for_threshold(
    threshold: NotNan<f64>,
    label_name: &String,
    label_validation: &LabelMetrics,
) -> Result<PrecisionAndRecall> {
    let threshold_index = label_validation
        .thresholds
        .iter()
        .position(|&val_threshold| val_threshold <= *threshold)
        .context(format!("Could not find threshold for label {}", label_name))?;

    let precision = NotNan::new(
        *label_validation
            .precisions
            .get(threshold_index)
            .context(format!("Could not get precision for label {}", label_name))?,
    )
    .unwrap();
    let recall = NotNan::new(
        *label_validation
            .recalls
            .get(threshold_index)
            .context(format!("Could not get recall for label {}", label_name))?,
    )
    .unwrap();

    Ok(PrecisionAndRecall { precision, recall })
}

#[derive(Clone)]
struct CompareConfig {
    validation_response: GetValidationResponse,
    dataset_name: DatasetFullName,
    model_version: ModelVersion,
}

impl CompareConfig {
    pub fn get_label_def(&self, label_name: &String) -> Result<Option<&LabelDef>> {
        let default_group = self
            .validation_response
            .label_groups
            .iter()
            .find(|group| group.name == "default")
            .context("Compare to dataset does not have a default label group")?;

        Ok(default_group
            .label_defs
            .iter()
            .find(|label| label.name == *label_name))
    }
}

fn get_compare_config(
    config: &Configuration,
    model_version: &Option<ModelVersion>,
    dataset_name: &Option<DatasetFullName>,
    stream_name: &StreamFullName,
) -> Result<Option<CompareConfig>> {
    if model_version.is_none() && dataset_name.is_none() {
        return Ok(None);
    }

    let dataset_name = if let Some(dataset_name) = dataset_name {
        dataset_name.clone()
    } else {
        format!("{}/{}", stream_name.owner, stream_name.dataset)
            .parse::<DatasetFullName>()
            .context("Failed to construct dataset full name from stream")?
    };

    let model_version = model_version
        .clone()
        .context("No compare to model version provided")?;

    info!("Getting validation for {}", dataset_name.0);
    let validation_response = get_validation(
        config,
        dataset_name.owner(),
        dataset_name.name(),
        &model_version.0.to_string(),
        None,
    )
    .context("Failed to get validation")?;

    Ok(Some(CompareConfig {
        validation_response,
        dataset_name,
        model_version,
    }))
}

fn get_stream_stat(
    label_threshold: &TriggerLabelThreshold,
    stream_full_name: &StreamFullName,
    model: &TriggerUserModel,
    compare_config: &Option<CompareConfig>,
    config: &Configuration,
) -> Result<StreamStat> {
    let label_name = label_threshold.name.join(" > ");

    info!(
        "Getting label validation for {} in dataset {}",
        label_name, stream_full_name.dataset
    );
    let request = GetLabelValidationRequest::new(label_name.clone());
    let label_validation_response = get_label_validation(
        config,
        &stream_full_name.owner,
        &stream_full_name.dataset,
        &model.version.to_string(),
        request,
        None,
    )
    .context("Failed to get label validation")?;
    let label_validation = label_validation_response.label_validation;

    // Extract threshold - it should be present since we filter out None values in the caller
    let threshold_value = label_threshold
        .threshold
        .context("Label threshold should have a threshold value")?;
    let threshold = NotNan::new(threshold_value).context("Invalid threshold value")?;

    let PrecisionAndRecall { precision, recall } =
        get_precision_and_recall_for_threshold(threshold, &label_name.clone(), &label_validation)?;

    let mut stream_stat = StreamStat {
        label_name: label_name.clone(),
        threshold,
        precision,
        recall,
        compare_to_precision: None,
        compare_to_recall: None,
        maintain_recall_precision: None,
        maintain_recall_threshold: None,
        maintain_precision_recall: None,
        maintain_precision_threshold: None,
    };

    if let Some(ref compare_config) = compare_config {
        if compare_config.get_label_def(&label_name)?.is_some() {
            info!(
                "Getting label validation for {} in dataset {}",
                label_name, compare_config.dataset_name.0
            );
            let request = GetLabelValidationRequest::new(label_name.clone());
            let compare_to_label_validation_response = get_label_validation(
                config,
                compare_config.dataset_name.owner(),
                compare_config.dataset_name.name(),
                &compare_config.model_version.0.to_string(),
                request,
                None,
            )
            .context("Failed to get comparison label validation")?;
            let compare_to_label_validation = compare_to_label_validation_response.label_validation;

            // Extract threshold - it should be present since we filter out None values in the caller
            let threshold_value = label_threshold
                .threshold
                .context("Label threshold should have a threshold value")?;
            let threshold = NotNan::new(threshold_value).context("Invalid threshold value")?;

            let same_threshold_precision_and_recall = get_precision_and_recall_for_threshold(
                threshold,
                &label_name,
                &compare_to_label_validation,
            )?;

            let maintain_recall_threshold_and_precision = get_threshold_and_precision_for_recall(
                recall,
                &label_name,
                &compare_to_label_validation,
            )
            .unwrap_or_default();

            let maintain_precision_threshold_and_recall = get_threshold_and_recall_for_precision(
                precision,
                &label_name,
                &compare_to_label_validation,
            )
            .unwrap_or_default();

            stream_stat.compare_to_precision = Some(same_threshold_precision_and_recall.precision);
            stream_stat.compare_to_recall = Some(same_threshold_precision_and_recall.recall);
            stream_stat.maintain_recall_precision =
                maintain_recall_threshold_and_precision.precision;
            stream_stat.maintain_recall_threshold =
                maintain_recall_threshold_and_precision.threshold;
            stream_stat.maintain_precision_recall = maintain_precision_threshold_and_recall.recall;
            stream_stat.maintain_precision_threshold =
                maintain_precision_threshold_and_recall.threshold;
        }
    }
    Ok(stream_stat)
}

/// Get validation statistics for a stream with optional comparison
pub fn get_stream_stats(
    config: &Configuration,
    args: &GetStreamStatsArgs,
    printer: &Printer,
    pool: &mut Pool,
) -> Result<()> {
    let GetStreamStatsArgs {
        stream_full_name,
        compare_to_model_version,
        compare_to_dataset,
    } = args;

    validate_stream_stats_args(compare_to_dataset, compare_to_model_version)?;

    let stream = get_stream_and_model(config, stream_full_name)?;
    let model = stream
        .model
        .as_ref()
        .context("No model associated with stream.")?;

    let compare_config = get_compare_config(
        config,
        compare_to_model_version,
        compare_to_dataset,
        stream_full_name,
    )?;

    let stream_stats =
        collect_stream_statistics_parallel(config, stream_full_name, model, &compare_config, pool)?;

    printer.print_resources(&stream_stats)?;
    Ok(())
}

/// Validate arguments for stream stats command
fn validate_stream_stats_args(
    compare_to_dataset: &Option<DatasetFullName>,
    compare_to_model_version: &Option<ModelVersion>,
) -> Result<()> {
    if compare_to_dataset.is_some() && compare_to_model_version.is_none() {
        return Err(anyhow!(
            "You cannot provide `compare_to_dataset` without `compare_to_model_version`"
        ));
    }
    Ok(())
}

/// Get stream and ensure it has a model
fn get_stream_and_model(
    config: &Configuration,
    stream_full_name: &StreamFullName,
) -> Result<Trigger> {
    info!("Getting stream {}", stream_full_name);
    let stream_response = get_stream_by_name(
        config,
        &stream_full_name.owner,
        &stream_full_name.dataset,
        &stream_full_name.stream,
    )
    .with_context(|| format!("Failed to get stream {}", stream_full_name))?;

    Ok(*stream_response.stream)
}

/// Collect stream statistics using parallel processing
fn collect_stream_statistics_parallel(
    config: &Configuration,
    stream_full_name: &StreamFullName,
    model: &TriggerUserModel,
    compare_config: &Option<CompareConfig>,
    pool: &mut Pool,
) -> Result<Vec<StreamStat>> {
    let label_thresholds = match &model.label_thresholds {
        Some(thresholds) => thresholds,
        None => {
            info!(
                "No label thresholds found for model version {}",
                model.version
            );
            return Ok(Vec::new());
        }
    };

    let (sender, receiver) = channel();

    pool.scoped(|scope| {
        for label_threshold in label_thresholds {
            if should_skip_threshold(label_threshold) {
                continue;
            }

            let sender = sender.clone();
            let model = model.clone();
            let compare_config = compare_config.clone();

            scope.execute(move || {
                let result = get_stream_stat(
                    label_threshold,
                    stream_full_name,
                    &model,
                    &compare_config,
                    config,
                );
                sender
                    .send(result)
                    .expect("Failed to send stream stat result");
            });
        }
    });

    drop(sender);
    let results: Vec<Result<StreamStat>> = receiver.iter().collect();

    let mut stream_stats = Vec::new();
    for result in results {
        stream_stats.push(result?);
    }

    stream_stats.sort_by(|a, b| a.label_name.cmp(&b.label_name));
    Ok(stream_stats)
}

/// Check if a label threshold should be skipped
fn should_skip_threshold(label_threshold: &TriggerLabelThreshold) -> bool {
    match label_threshold.threshold {
        Some(threshold) if threshold >= 1.0 => {
            // Precision and recall will always be 0
            true
        }
        None => {
            info!(
                "Skipping label threshold {:?} - no threshold value",
                label_threshold.name
            );
            true
        }
        _ => false,
    }
}

/// Fetch comments from a stream with optional continuous polling
pub fn get_stream_comments(config: &Configuration, args: &GetStreamCommentsArgs) -> Result<()> {
    let GetStreamCommentsArgs {
        stream,
        size,
        listen,
        individual_advance,
    } = args;

    match listen {
        Some(delay) => {
            fetch_stream_comments_continuously(config, stream, *size, *delay, *individual_advance)
        }
        None => fetch_stream_comments_once(config, stream, *size),
    }
}

/// Fetch stream comments once and return the batch
fn fetch_stream_comments_once(
    config: &Configuration,
    stream: &StreamFullName,
    batch_size: u32,
) -> Result<()> {
    let request = FetchFromStreamRequest::new(batch_size as i32);
    let batch = fetch_from_stream(
        config,
        &stream.owner,
        &stream.dataset,
        &stream.stream,
        request,
    )
    .with_context(|| format!("Failed to fetch comments from stream {}", stream))?;

    print_resources_as_json(Some(&batch), io::stdout().lock())
}

/// Continuously poll stream for new comments
fn fetch_stream_comments_continuously(
    config: &Configuration,
    stream: &StreamFullName,
    batch_size: u32,
    delay_seconds: f64,
    individual_advance: bool,
) -> Result<()> {
    loop {
        let request = FetchFromStreamRequest::new(batch_size as i32);
        let batch = fetch_from_stream(
            config,
            &stream.owner,
            &stream.dataset,
            &stream.stream,
            request,
        )
        .with_context(|| format!("Failed to fetch comments from stream {}", stream))?;

        if batch.results.is_empty() {
            handle_empty_batch(config, stream, &batch, delay_seconds)?;
            continue;
        }

        process_batch_results(config, stream, &batch, individual_advance)?;
    }
}

/// Handle the case when a batch is empty
fn handle_empty_batch(
    config: &Configuration,
    stream: &StreamFullName,
    batch: &openapi::models::FetchFromStreamResponse,
    delay_seconds: f64,
) -> Result<()> {
    if batch.filtered == 0 {
        // No new comments, wait before polling again
        std::thread::sleep(std::time::Duration::from_secs_f64(delay_seconds));
    } else {
        // Comments were filtered out, advance the stream
        let advance_request = AdvanceStreamRequest::new(batch.sequence_id.clone());
        advance_stream(
            config,
            &stream.owner,
            &stream.dataset,
            &stream.stream,
            advance_request,
        )
        .with_context(|| format!("Failed to advance stream {} for empty batch", stream))?;
    }
    Ok(())
}

/// Process and output batch results
fn process_batch_results(
    config: &Configuration,
    stream: &StreamFullName,
    batch: &openapi::models::FetchFromStreamResponse,
    individual_advance: bool,
) -> Result<()> {
    let needs_final_advance =
        !individual_advance || batch.sequence_id != batch.results.last().unwrap().sequence_id;

    for result in &batch.results {
        print_resources_as_json(Some(result), io::stdout().lock())?;

        if individual_advance {
            let advance_request = AdvanceStreamRequest::new(result.sequence_id.clone());
            advance_stream(
                config,
                &stream.owner,
                &stream.dataset,
                &stream.stream,
                advance_request,
            )
            .with_context(|| {
                format!("Failed to advance stream {} for individual comment", stream)
            })?;
        }
    }

    if needs_final_advance {
        let advance_request = AdvanceStreamRequest::new(batch.sequence_id.clone());
        advance_stream(
            config,
            &stream.owner,
            &stream.dataset,
            &stream.stream,
            advance_request,
        )
        .with_context(|| format!("Failed to advance stream {} for batch", stream))?;
    }

    Ok(())
}
