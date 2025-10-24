use crate::utils::{
    types::identifiers::{DatasetIdentifier, SourceIdentifier},
    DatasetFullName,
};
use crate::{
    commands::{
        create::annotations::{upload_batch_of_annotations, CommentIdComment, NewAnnotation},
        parse::{get_progress_bar, upload_batch_of_comments},
    },
    parse::Statistics,
};
use anyhow::{Context, Result};
use log::{error, info};
use openapi::{
    apis::{
        configuration::Configuration,
        datasets_api::get_dataset,
        sources_api::{get_source, get_source_by_id},
    },
    models::{
        group_labellings_request::Group, CommentNew, GroupLabellingsRequest, Label, LabelSentiment,
        Message, MessageRichText, Source,
    },
};
use scoped_threadpool::Pool;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::{mpsc::channel, Arc};
use structopt::StructOpt;

const UPLOAD_BATCH_SIZE: usize = 4;

#[derive(Debug, StructOpt)]
pub struct ParseAicClassificationCsvArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to the csv to parse
    file_path: PathBuf,

    #[structopt(short = "s", long = "source")]
    /// The source to upload the data to
    source: SourceIdentifier,

    #[structopt(short = "d", long = "dataset")]
    /// The dataset to upload annotations to
    dataset: DatasetIdentifier,

    #[structopt(short = "n", long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,
}

#[derive(Deserialize)]
pub struct AicClassificationRecord {
    input: String,
    target: String,
}

#[allow(clippy::too_many_arguments)]
fn send_comments_if_needed(
    comments: &mut Vec<CommentNew>,
    annotations: &mut Vec<NewAnnotation>,
    force_send: bool,
    pool: &mut Pool,
    config: &Configuration,
    source: &Source,
    statistics: &Statistics,
    dataset: &DatasetFullName,
    no_charge: bool,
) -> Result<()> {
    let thread_count = pool.thread_count();
    let should_upload = comments.len() > (thread_count as usize * UPLOAD_BATCH_SIZE);

    if !force_send && !should_upload {
        return Ok(());
    }

    let chunks: Vec<_> = comments.chunks(UPLOAD_BATCH_SIZE).collect();

    let (error_sender, error_receiver) = channel();
    pool.scoped(|scope| {
        for chunk in chunks {
            scope.execute(|| {
                let result = upload_batch_of_comments(config, source, chunk, no_charge, statistics);

                if let Err(error) = result {
                    error_sender.send(error).expect("Could not send error");
                }
            });
        }
    });

    upload_batch_of_annotations(
        annotations,
        config,
        source,
        statistics,
        dataset,
        pool,
        false,
    )?;

    if let Ok(error) = error_receiver.try_recv() {
        Err(error)
    } else {
        comments.clear();
        annotations.clear();
        Ok(())
    }
}

pub fn parse(
    config: &Configuration,
    args: &ParseAicClassificationCsvArgs,
    pool: &mut Pool,
) -> Result<()> {
    let ParseAicClassificationCsvArgs {
        file_path,
        source,
        dataset,
        no_charge,
    } = args;

    let source = match source {
        SourceIdentifier::Id(source_id) => {
            let response =
                get_source_by_id(config, source_id).context("Failed to get source by ID")?;
            response.source
        }
        SourceIdentifier::FullName(full_name) => {
            let response = get_source(config, full_name.owner(), full_name.name())
                .context("Failed to get source by name")?;
            response.source
        }
    };

    let dataset = match dataset {
        DatasetIdentifier::Id(_) => {
            anyhow::bail!(
                "Dataset lookup by ID is not supported. Please use dataset full name (owner/name)"
            )
        }
        DatasetIdentifier::FullName(full_name) => {
            let response = get_dataset(config, full_name.owner(), full_name.name())
                .context("Failed to get dataset")?;
            response.dataset
        }
    };
    let record_count = csv::Reader::from_path(file_path)?.records().count();

    let statistics = Arc::new(Statistics::new());
    let _progress = get_progress_bar(record_count as u64, &statistics);

    let mut reader = csv::Reader::from_path(file_path)?;

    let headers = reader.headers()?.clone();

    let mut comments: Vec<CommentNew> = Vec::new();
    let mut annotations: Vec<NewAnnotation> = Vec::new();
    for (idx, row) in reader.records().enumerate() {
        match row {
            Ok(row) => {
                let record: AicClassificationRecord = row.deserialize(Some(&headers))?;
                let comment_id_str = idx.to_string();

                comments.push(CommentNew {
                    id: comment_id_str.clone(),
                    timestamp: Some(chrono::Utc::now().to_rfc3339()),
                    messages: Some(vec![Message {
                        body: Box::new(MessageRichText::new(record.input)),
                        ..Default::default()
                    }]),
                    ..Default::default()
                });
                annotations.push(NewAnnotation {
                    comment: CommentIdComment { id: comment_id_str },
                    labelling: Some(vec![GroupLabellingsRequest {
                        group: Some(Group::Default),
                        assigned: Some(vec![Label {
                            name: record.target,
                            sentiment: LabelSentiment::Positive,
                            metadata: None,
                            instructions: None,
                        }]),
                        dismissed: None,
                        uninformative: None,
                    }]),
                    entities: None,
                    moon_forms: None,
                });

                send_comments_if_needed(
                    &mut comments,
                    &mut annotations,
                    false,
                    pool,
                    config,
                    &source,
                    &statistics,
                    &format!("{}/{}", dataset.owner, dataset.name).parse::<DatasetFullName>()?,
                    *no_charge,
                )?;
                statistics.increment_processed()
            }
            Err(_) => {
                error!("Failed to process row {idx}");
                statistics.increment_failed();
                statistics.increment_processed()
            }
        }
    }
    send_comments_if_needed(
        &mut comments,
        &mut annotations,
        true,
        pool,
        config,
        &source,
        &statistics,
        &format!("{}/{}", dataset.owner, dataset.name).parse::<DatasetFullName>()?,
        *no_charge,
    )?;

    info!(
        "Uploaded {}. {} Failed",
        statistics.num_uploaded(),
        statistics.num_failed()
    );
    Ok(())
}
