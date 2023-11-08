use anyhow::{anyhow, Context, Result};
use colored::{ColoredString, Colorize};
use log::info;
use ordered_float::NotNan;
use prettytable::row;
use reinfer_client::resources::stream::{StreamLabelThreshold, StreamModel};
use reinfer_client::resources::validation::ValidationResponse;
use reinfer_client::{
    resources::validation::LabelValidation, Client, DatasetIdentifier, ModelVersion, StreamFullName,
};
use reinfer_client::{DatasetFullName, LabelDef, LabelName};
use scoped_threadpool::Pool;
use serde::Serialize;
use std::sync::mpsc::channel;
use std::{
    fs::File,
    io,
    io::{BufWriter, Write},
    path::PathBuf,
};
use structopt::StructOpt;

use crate::printer::{print_resources_as_json, DisplayTable, Printer};

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

pub fn get(client: &Client, args: &GetStreamsArgs, printer: &Printer) -> Result<()> {
    let GetStreamsArgs { dataset, path } = args;

    let file: Option<Box<dyn Write>> = match path {
        Some(path) => Some(Box::new(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?,
        )),
        None => None,
    };

    let dataset_name = client
        .get_dataset(dataset.clone())
        .context("Operation to get dataset has failed.")?
        .full_name();
    let mut streams = client
        .get_streams(&dataset_name)
        .context("Operation to list streams has failed.")?;
    streams.sort_unstable_by(|lhs, rhs| lhs.name.0.cmp(&rhs.name.0));

    if let Some(file) = file {
        print_resources_as_json(streams, file)
    } else {
        printer.print_resources(&streams)
    }
}

#[derive(Serialize)]
pub struct StreamStat {
    label_name: LabelName,
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
            self.label_name.0,
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
                format!("{:.5}", threshold).normal()
            } else {
                "none".dimmed()
            },
            if let Some(threshold) = self.maintain_precision_threshold {
                format!("{:.5}", threshold).normal()
            } else {
                "none".dimmed()
            }
        ]
    }
}

fn red_if_lower_green_otherwise(test: NotNan<f64>, threshold: NotNan<f64>) -> ColoredString {
    let test_str = format!("{:.3}", test);

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
    label_name: &LabelName,
    label_validation: &LabelValidation,
) -> Result<ThresholdAndPrecision> {
    let recall_index = label_validation
        .recalls
        .iter()
        .position(|&val_recall| val_recall >= recall)
        .context(format!("Could not get recall for label {}", label_name.0))?;

    let precision = label_validation.precisions.get(recall_index);

    let threshold = label_validation.thresholds.get(recall_index);

    Ok(ThresholdAndPrecision {
        threshold: threshold.cloned(),
        precision: precision.cloned(),
    })
}

#[derive(Default)]
struct ThresholdAndRecall {
    threshold: Option<NotNan<f64>>,
    recall: Option<NotNan<f64>>,
}

fn get_threshold_and_recall_for_precision(
    precision: NotNan<f64>,
    label_name: &LabelName,
    label_validation: &LabelValidation,
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
        label_name.0
    ))?;

    let recall = label_validation.recalls.get(precision_index);
    let threshold = label_validation.thresholds.get(precision_index);

    Ok(ThresholdAndRecall {
        threshold: threshold.cloned(),
        recall: recall.cloned(),
    })
}

#[derive(Default)]
struct PrecisionAndRecall {
    precision: NotNan<f64>,
    recall: NotNan<f64>,
}

fn get_precision_and_recall_for_threshold(
    threshold: NotNan<f64>,
    label_name: &LabelName,
    label_validation: &LabelValidation,
) -> Result<PrecisionAndRecall> {
    let threshold_index = label_validation
        .thresholds
        .iter()
        .position(|&val_threshold| val_threshold <= threshold)
        .context(format!(
            "Could not find threshold for label {}",
            label_name.0
        ))?;

    let precision = *label_validation
        .precisions
        .get(threshold_index)
        .context(format!(
            "Could not get precision for label {}",
            label_name.0
        ))?;
    let recall = *label_validation
        .recalls
        .get(threshold_index)
        .context(format!("Could not get recall for label {}", label_name.0))?;

    Ok(PrecisionAndRecall { precision, recall })
}

#[derive(Clone)]
struct CompareConfig {
    validation: ValidationResponse,
    dataset_name: DatasetFullName,
    model_version: ModelVersion,
}

impl CompareConfig {
    pub fn get_label_def(&self, label_name: &LabelName) -> Result<Option<&LabelDef>> {
        Ok(self
            .validation
            .get_default_label_group()
            .context("Compare to dataset does not have a default label group")?
            .label_defs
            .iter()
            .find(|label| label.name == *label_name))
    }
}

fn get_compare_config(
    client: &Client,
    model_version: &Option<ModelVersion>,
    dataset_name: &Option<DatasetFullName>,
    stream_name: &StreamFullName,
) -> Result<Option<CompareConfig>> {
    if model_version.is_none() && dataset_name.is_none() {
        return Ok(None);
    }

    let dataset_name = if let Some(dataset_name) = dataset_name {
        dataset_name
    } else {
        &stream_name.dataset
    };

    let model_version = model_version
        .clone()
        .context("No compare to model version provided")?;

    info!("Getting validation for {}", dataset_name.0);
    let validation = client.get_validation(dataset_name, &model_version)?;

    Ok(Some(CompareConfig {
        validation,
        dataset_name: dataset_name.clone(),
        model_version,
    }))
}

fn get_stream_stat(
    label_threshold: &StreamLabelThreshold,
    stream_full_name: &StreamFullName,
    model: &StreamModel,
    compare_config: &Option<CompareConfig>,
    client: &Client,
) -> Result<StreamStat> {
    let label_name = reinfer_client::LabelName(label_threshold.name.join(" > "));

    info!(
        "Getting label validation for {} in dataset {}",
        label_name.0, stream_full_name.dataset.0
    );
    let label_validation =
        client.get_label_validation(&label_name, &stream_full_name.dataset, &model.version)?;

    let PrecisionAndRecall { precision, recall } = get_precision_and_recall_for_threshold(
        label_threshold.threshold,
        &label_name.clone(),
        &label_validation,
    )?;

    let mut stream_stat = StreamStat {
        label_name: label_name.clone(),
        threshold: label_threshold.threshold,
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
                label_name.0, compare_config.dataset_name.0
            );
            let compare_to_label_validation = client.get_label_validation(
                &label_name,
                &compare_config.dataset_name,
                &compare_config.model_version,
            )?;

            let same_threshold_precision_and_recall = get_precision_and_recall_for_threshold(
                label_threshold.threshold,
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

pub fn get_stream_stats(
    client: &Client,
    args: &GetStreamStatsArgs,
    printer: &Printer,
    pool: &mut Pool,
) -> Result<()> {
    let GetStreamStatsArgs {
        stream_full_name,
        compare_to_model_version,
        compare_to_dataset,
    } = args;

    if compare_to_dataset.is_some() && compare_to_model_version.is_none() {
        return Err(anyhow!(
            "You cannot provide `compare_to_dataset` without `compare_to_model_version`"
        ));
    }

    info!("Getting Stream");
    let stream = client.get_stream(stream_full_name)?;
    let model = stream.model.context("No model associated with stream.")?;

    let compare_config = get_compare_config(
        client,
        compare_to_model_version,
        compare_to_dataset,
        stream_full_name,
    )?;

    let mut stream_stats = Vec::new();

    let (sender, receiver) = channel();

    pool.scoped(|scope| {
        for label_threshold in &model.label_thresholds {
            if label_threshold.threshold >= NotNan::new(1.0).expect("Could not create NotNan") {
                // As the precision and recall will always be 0
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
                    client,
                );
                sender.send(result).expect("Could not send result");
            });
        }
    });

    drop(sender);
    let results: Vec<Result<StreamStat>> = receiver.iter().collect();

    for result in results {
        let stream_stat = result?;
        stream_stats.push(stream_stat)
    }

    stream_stats.sort_by(|a, b| a.label_name.0.cmp(&b.label_name.0));

    printer.print_resources(&stream_stats)?;
    Ok(())
}

pub fn get_stream_comments(client: &Client, args: &GetStreamCommentsArgs) -> Result<()> {
    let GetStreamCommentsArgs {
        stream,
        size,
        listen,
        individual_advance,
    } = args;

    match listen {
        Some(delay) => loop {
            let batch = client
                .fetch_stream_comments(stream, *size)
                .context("Operation to fetch stream comments failed.")?;
            if batch.results.is_empty() {
                if batch.filtered == 0 {
                    std::thread::sleep(std::time::Duration::from_secs_f64(*delay));
                } else {
                    client
                        .advance_stream(stream, batch.sequence_id)
                        .context("Operation to advance stream for batch failed.")?;
                }
                continue;
            }
            let needs_final_advance = !individual_advance
                || batch.sequence_id != batch.results.last().unwrap().sequence_id;
            for result in batch.results {
                print_resources_as_json(Some(&result), io::stdout().lock())?;

                if *individual_advance {
                    client
                        .advance_stream(stream, result.sequence_id)
                        .context("Operation to advance stream for comment failed.")?;
                }
            }
            if needs_final_advance {
                client
                    .advance_stream(stream, batch.sequence_id)
                    .context("Operation to advance stream for batch failed.")?;
            }
        },
        None => {
            let batch = client
                .fetch_stream_comments(stream, *size)
                .context("Operation to fetch stream comments failed.")?;
            print_resources_as_json(Some(&batch), io::stdout().lock())
        }
    }
}
