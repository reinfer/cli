use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::channel,
        Arc,
    },
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use itertools::Itertools;
use scoped_threadpool::Pool;
use structopt::StructOpt;

use openapi::{
    apis::{
        buckets_api::{create_bucket, get_bucket},
        comments_api::add_comments,
        configuration::Configuration,
        datasets_api::{
            create_dataset, get_all_datasets, get_dataset, update_comment_labelling, update_dataset,
        },
        emails_api::add_emails_to_bucket,
        label_defs_api::create_label_defs_bulk,
        projects_api::get_project,
        sources_api::{create_source, get_source, get_source_by_id},
    },
    models::{
        bucket_new::BucketType, AddCommentsRequest, AddEmailsToBucketRequest, Bucket, BucketNew,
        BuiltinLabelDefRequest, CommentNew, CreateBucketRequest, CreateDatasetRequest,
        CreateIxpDatasetRequest, CreateOrUpdateLabelDefsBulkRequest, CreateSourceRequest, Dataset,
        DatasetFlag, DatasetNew, DatasetUpdate, EntityDefNew, EntityRuleSetNewApi,
        FieldChoiceNewApi, GeneralFieldDefNew, InheritsFrom, IxpDatasetNew, LabelDef, LabelDefNew,
        ModelConfig, Source, SourceKind, SourceUpdate, UpdateCommentLabellingRequest,
        UpdateDatasetRequest,
    },
};

use crate::utils::{
    add_emails_to_bucket_with_split_on_failure, conversions::HasAnnotations,
    handle_split_on_failure_result, sync_comments_with_split_on_failure, upload_ixp_document_bytes,
    AttachmentExt, FullName as DatasetFullName, NewAnnotatedComment, ProjectName, SourceId,
    DEFAULT_LABEL_GROUP_NAME,
};

// Type aliases for compatibility
pub type BucketId = String;

use crate::{
    commands::{
        create::{
            annotations::{
                upload_batch_of_annotations, AnnotationStatistic, CommentIdComment, NewAnnotation,
            },
            project::create_project_with_wait,
        },
        ensure_uip_user_consents_to_ai_unit_charge,
        package::{AttachmentKey, CommentBatchKey, EmailBatchKey, Package},
    },
    progress::Progress,
    utils::{auth::refresh::refresh_user_permissions, get_current_user},
};

const MAX_UNPACK_CM_ATTEMPTS: u64 = 20;
const UNPACK_CM_SLEEP_SECONDS: u64 = 3;

/// Helper function to parse dataset full name into owner and name parts
fn parse_dataset_full_name(dataset_full_name: &str) -> Result<(&str, &str)> {
    let parts: Vec<&str> = dataset_full_name.split('/').collect();
    if parts.len() == 2 {
        Ok((parts[0], parts[1]))
    } else {
        Err(anyhow!(
            "Invalid dataset full name format: {}",
            dataset_full_name
        ))
    }
}

#[derive(Debug, StructOpt)]
pub struct UploadPackageArgs {
    /// Path of the package to upload
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    #[structopt(long = "resume-on-error")]
    /// Whether to attempt to resume processing on error
    resume_on_error: bool,

    #[structopt(long = "max-attachment-memory-mb", default_value = "256")]
    /// Maximum number of mb to use to store attachments in memory
    max_attachment_memory_mb: u64,

    #[structopt(long = "dataset-creation-timeout", default_value = "30")]
    /// Maximum number of seconds to wait for dataset to be created
    dataset_creation_timeout: u64,

    #[structopt(long = "name", short = "n")]
    /// The name of the new project
    new_project_name: Option<DatasetOrOwnerName>,

    #[structopt(long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,

    #[structopt(short = "y", long = "yes")]
    /// Consent to ai unit charge. Suppresses confirmation prompt.
    yes: bool,

    #[structopt(long = "skip-comment-upload")]
    /// Don't upload comments for CM packages
    skip_comment_upload: bool,
}

fn wait_for_dataset_to_exist(
    dataset: &Dataset,
    config: &Configuration,
    timeout_s: u64,
) -> Result<()> {
    let start_time = Instant::now();

    while (start_time - Instant::now()).as_secs() <= timeout_s {
        refresh_user_permissions(config)?;
        let all_datasets_response =
            get_all_datasets(config).context("Failed to get all datasets")?;
        let datasets = all_datasets_response.datasets;

        let dataset_exists = datasets
            .iter()
            .map(|ds| DatasetFullName(format!("{}/{}", ds.owner, ds.name)))
            .contains(&DatasetFullName(format!(
                "{}/{}",
                dataset.owner, dataset.name
            )));

        if dataset_exists {
            return Ok(());
        } else {
            sleep(Duration::from_millis(500));
        }
    }

    Err(anyhow!("Timeout waiting for dataset to be created"))
}

fn create_ixp_dataset(
    name: String,
    label_defs: Vec<LabelDef>,
    config: &Configuration,
    timeout_s: u64,
    model_config: ModelConfig,
    entity_defs: Vec<EntityDefNew>,
) -> Result<Dataset> {
    let mut new_label_defs = Vec::new();

    label_defs.iter().try_for_each(|label| -> Result<()> {
        let new_label = LabelDefNew {
            title: label.title.clone(),
            instructions: label.instructions.clone(),
            external_id: label.external_id.clone(),
            pretrained: None, // Set to None for now
            trainable: label.trainable.map(|t| Some(t)),
            name: label.name.clone(),
            moon_form: label.moon_form.clone().map(|forms| {
                forms
                    .into_iter()
                    .map(|form| openapi::models::MoonFormFieldDefNew {
                        field_type_id: Some(Some(form.field_type_id.clone())),
                        id: Some(Some(form.id.clone())),
                        kind: form.kind,
                        name: form.name,
                        instructions: form.instructions,
                        rule_set: Some(None),
                    })
                    .collect()
            }),
        };
        new_label_defs.push(new_label);
        Ok(())
    })?;

    let create_request = CreateIxpDatasetRequest {
        dataset: Box::new(IxpDatasetNew { name }),
    };
    let create_response =
        openapi::apis::ixp_datasets_api::create_ixp_dataset(config, create_request)?;
    let dataset = create_response.dataset;

    wait_for_dataset_to_exist(&dataset, config, timeout_s)?;

    let dataset_full_name = DatasetFullName(format!("{}/{}", dataset.owner, dataset.name));
    let (owner, dataset_name) = parse_dataset_full_name(&dataset_full_name)?;

    let update_request = UpdateDatasetRequest {
        dataset: Box::new(DatasetUpdate {
            _model_config: Some(Box::new(model_config)),
            source_ids: None,
            title: None,
            description: None,
            entity_defs: Some(
                entity_defs
                    .into_iter()
                    .map(|e| {
                        openapi::models::DatasetUpdateEntityDefsInner::new(
                            e.id.unwrap_or_default(),
                            e.name.clone(),
                            e.title.clone(),
                            openapi::models::InheritsFrom::new(),
                            e.trainable,
                        )
                    })
                    .collect(),
            ),
            ..Default::default()
        }),
    };
    update_dataset(config, owner, dataset_name, update_request)?;

    let create_labels_request = CreateOrUpdateLabelDefsBulkRequest {
        label_defs: new_label_defs
            .into_iter()
            .map(|l| {
                openapi::models::CreateOrUpdateLabelDefsBulkRequestLabelDefsInner::new(
                    l.title
                        .unwrap_or(l.external_id.unwrap_or("Unnamed".to_string())),
                )
            })
            .collect(),
    };
    create_label_defs_bulk(
        config,
        owner,
        dataset_name,
        DEFAULT_LABEL_GROUP_NAME,
        create_labels_request,
    )
    .context("Error when creating label defs")?;
    Ok(*dataset)
}

pub struct IxpDatasetSources {
    runtime: Source,
    design_time: Source,
}

enum SourceProvider<'a> {
    Packaged(&'a mut Package),
    Remote(&'a Configuration),
}

impl SourceProvider<'_> {
    fn get_source(&mut self, source_id: &SourceId) -> Result<Source> {
        match self {
            Self::Packaged(package) => package.get_source_by_id(source_id.as_ref()),
            Self::Remote(config) => {
                let response = get_source_by_id(config, source_id.as_ref())
                    .context("Failed to get source by ID")?;
                Ok(*response.source)
            }
        }
    }
}

impl Display for SourceProvider<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Packaged(_) => "packaged",
                Self::Remote(_) => "remote",
            }
        )
    }
}

fn get_ixp_source(dataset: &Dataset, mut provider: SourceProvider) -> Result<IxpDatasetSources> {
    let mut source_by_kind: HashMap<SourceKind, Source> = HashMap::new();
    dataset
        .source_ids
        .iter()
        .try_for_each(|source_id| -> Result<()> {
            let source = provider
                .get_source(&SourceId(source_id.clone()))
                .context("Could not get source")?;
            if source_by_kind.contains_key(&source._kind) {
                return Err(anyhow!("Duplicate source kind found in packaged dataset"));
            }

            source_by_kind.insert(source._kind.clone(), source.clone());
            Ok(())
        })?;

    let runtime = source_by_kind
        .get(&SourceKind::IxpRuntime)
        .context(anyhow!(
            "Could not find runtime source for {0} dataset {1}",
            provider,
            dataset.id
        ))?
        .clone();

    let design_time = source_by_kind
        .get(&SourceKind::IxpDesign)
        .context(anyhow!(
            "Could not find design time source for {0} dataset {1}",
            provider,
            dataset.id
        ))?
        .clone();

    Ok(IxpDatasetSources {
        runtime,
        design_time,
    })
}

#[allow(clippy::too_many_arguments)]
fn unpack_ixp_source(
    old_source: &Source,
    new_source: &Source,
    new_dataset: &Dataset,
    package: &mut Package,
    config: &Configuration,
    pool: &mut Pool,
    resume_on_error: bool,
    statistics: &IxpStatistics,
    max_attachment_memory_mb: &u64,
) -> Result<()> {
    let batch_count = package.get_comment_batch_count_for_source(&old_source.id);

    let mut documents = Vec::new();
    for idx in 0..batch_count {
        let batch = package
            .get_comment_batch(&old_source.id, CommentBatchKey(idx))
            .context("Could not get comment batch")?;

        batch.iter().try_for_each(|comment| -> Result<()> {
            comment.comment.attachments.iter().enumerate().try_for_each(
                |(idx, attachment)| -> Result<()> {
                    let read_result = package.read_document(
                        &old_source.id,
                        &comment.comment.id,
                        &AttachmentKey(idx),
                        attachment.extension(),
                    );

                    match read_result {
                        Ok(result) => {
                            statistics.add_document_reads(1);

                            documents.push(AnnotationWithContent {
                                comment: crate::utils::conversions::convert_annotated_comment(
                                    comment.clone(),
                                ),
                                content: result,
                                size: attachment.size as usize,
                                file_name: attachment.name.clone(),
                            });

                            if should_upload_batch(&documents, max_attachment_memory_mb) {
                                upload_batch(
                                    &documents,
                                    config,
                                    &SourceId(new_source.id.clone()),
                                    &DatasetFullName(format!(
                                        "{}/{}",
                                        new_dataset.owner, new_dataset.name
                                    )),
                                    resume_on_error,
                                    statistics,
                                    pool,
                                )
                                .context("Failed to upload batch")?;
                                documents.clear();
                            }
                            Ok(())
                        }
                        Err(_) if resume_on_error => {
                            statistics.add_failed_document_reads(1);
                            Ok(())
                        }
                        Err(err) => Err(err),
                    }
                },
            )
        })?;
    }
    upload_batch(
        &documents,
        config,
        &SourceId(new_source.id.clone()),
        &DatasetFullName(format!("{}/{}", new_dataset.owner, new_dataset.name)),
        resume_on_error,
        statistics,
        pool,
    )
}

fn should_upload_batch(batch: &[AnnotationWithContent], max_attachment_memory_mb: &u64) -> bool {
    batch
        .iter()
        .map(|attachment| attachment.size as u64)
        .sum::<u64>()
        / 1_000_000
        >= *max_attachment_memory_mb
}

struct AnnotationWithContent {
    comment: NewAnnotatedComment,
    file_name: String,
    content: Vec<u8>,
    size: usize,
}

fn upload_batch(
    batch: &Vec<AnnotationWithContent>,
    config: &Configuration,
    source_id: &SourceId,
    dataset_name: &DatasetFullName,
    resume_on_error: bool,
    statistics: &IxpStatistics,
    pool: &mut Pool,
) -> Result<()> {
    let (error_sender, error_receiver) = channel();

    pool.scoped(|scope| {
        for item in batch {
            let error_sender = error_sender.clone();
            scope.execute(move || {
                let upload_result = upload_ixp_document_bytes(
                    config,
                    &source_id.to_string(),
                    "latest",
                    Some(item.content.clone()),
                    Some(item.file_name.clone()),
                )
                .map(|response| response.document.id)
                .map_err(anyhow::Error::msg);

                match upload_result {
                    Ok(new_comment_id) => {
                        statistics.add_document_uploads(1);

                        let comment_uid = format!("{}.{}", source_id, new_comment_id);

                        if item.comment.has_annotations() {
                            let dataset_full_name = dataset_name;
                            let parts: Vec<&str> = dataset_full_name.as_str().split('/').collect();
                            let (owner, dataset_name_part) = if parts.len() == 2 {
                                (parts[0], parts[1])
                            } else {
                                error_sender
                                    .send(anyhow!(
                                        "Invalid dataset full name format: {}",
                                        dataset_full_name.as_str()
                                    ))
                                    .unwrap();
                                return;
                            };

                            let update_request = UpdateCommentLabellingRequest {
                                context: None,
                                labelling: item.comment.labelling.clone(),
                                entities: item.comment.entities.clone(),
                                moon_forms: item.comment.moon_forms.clone(),
                            };

                            let labelling_result = update_comment_labelling(
                                config,
                                owner,
                                dataset_name_part,
                                &comment_uid,
                                update_request,
                            );

                            match labelling_result {
                                Ok(_) => {
                                    statistics.add_annotation(1);
                                }
                                Err(_) if resume_on_error => {
                                    statistics.add_failed_annotations(1);
                                }
                                Err(err) => error_sender
                                    .send(anyhow::Error::msg(err))
                                    .expect("Could not send error"),
                            }
                        }
                    }

                    Err(_) if resume_on_error => {
                        statistics.add_failed_uploads(1);
                    }

                    Err(err) => error_sender.send(err).expect("Could not send error"),
                }
            })
        }
    });

    drop(error_sender);

    if let Ok(error) = error_receiver.try_recv() {
        return Err(anyhow!("Failed to upload document {}", error));
    };

    Ok(())
}

fn unpack_cm_bucket(
    config: &Configuration,
    packaged_bucket_id: &BucketId,
    package: &mut Package,
    resume_on_error: &bool,
    statistics: &Arc<CmStatistics>,
    no_charge: bool,
    new_project_name: Option<DatasetOrOwnerName>,
) -> Result<Bucket> {
    let mut packaged_bucket = package.get_bucket_by_id(packaged_bucket_id)?;

    if let Some(new_project_name) = new_project_name {
        packaged_bucket.owner = new_project_name.0.clone();
    }

    // Get Existing bucket, or create if it doesn't exist
    let bucket_full_name = format!("{}/{}", packaged_bucket.owner, packaged_bucket.name);
    let parts: Vec<&str> = bucket_full_name.split('/').collect();
    let (owner, bucket_name) = if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        return Err(anyhow!(
            "Invalid bucket full name format: {}",
            bucket_full_name
        ));
    };

    let bucket = match get_bucket(config, owner, bucket_name) {
        Ok(response) => *response.bucket,
        Err(_) => {
            let create_request = CreateBucketRequest {
                bucket: Box::new(BucketNew {
                    bucket_type: Some(BucketType::Emails),
                    title: None,
                }),
            };
            let create_response = create_bucket(config, owner, bucket_name, create_request)?;
            *create_response.bucket
        }
    };

    // Upload batches of emails to the bucket
    let email_batch_count = package.get_email_batch_count_for_bucket(&packaged_bucket.id);
    for idx in 0..email_batch_count {
        let batch = package
            .get_email_batch(&packaged_bucket.id, EmailBatchKey(idx))
            .context("Could not get email batch")?;

        let bucket_full_name = format!("{}/{}", bucket.owner, bucket.name);
        let parts: Vec<&str> = bucket_full_name.split('/').collect();
        let (owner, bucket_name) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            return Err(anyhow!(
                "Invalid bucket full name format: {}",
                bucket_full_name
            ));
        };

        let add_emails_request = AddEmailsToBucketRequest { emails: batch };

        if *resume_on_error {
            // Use cleaner wrapper function for split-on-failure
            let result = add_emails_to_bucket_with_split_on_failure(
                config,
                owner,
                bucket_name,
                add_emails_request.emails.clone(),
                Some(no_charge),
            )?;

            statistics.add_email_batch_uploads(1);
            statistics.add_failed_email_uploads(result.num_failed);
            handle_split_on_failure_result(
                result,
                add_emails_request.emails.len(),
                "add_emails_to_bucket",
            );
        } else {
            add_emails_to_bucket(
                config,
                owner,
                bucket_name,
                add_emails_request,
                Some(no_charge),
            )?;
            statistics.add_email_batch_uploads(1);
        }
    }
    Ok(bucket)
}

#[allow(clippy::too_many_arguments)]
fn unpack_cm_source(
    config: &Configuration,
    mut packaged_source_info: Source,
    bucket: Option<Bucket>,
    package: &mut Package,
    resume_on_error: &bool,
    no_charge: bool,
    statistics: &Arc<CmStatistics>,
    new_project_name: Option<DatasetOrOwnerName>,
    skip_comment_upload: bool,
) -> Result<Source> {
    // Get existing source or create a new one if it doesn't exist
    if let Some(project_name) = new_project_name {
        packaged_source_info.owner = project_name.0.clone();
    }

    let source_full_name = format!(
        "{}/{}",
        packaged_source_info.owner, packaged_source_info.name
    );
    let parts: Vec<&str> = source_full_name.split('/').collect();
    let (owner, source_name) = if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        return Err(anyhow!(
            "Invalid source full name format: {}",
            source_full_name
        ));
    };

    let source = match get_source(config, owner, source_name) {
        Ok(response) => *response.source,
        Err(_) => {
            let create_request = CreateSourceRequest {
                source: Box::new(SourceUpdate {
                    bucket_id: bucket.map(|b| Some(b.id)),
                    description: Some(packaged_source_info.description.clone()),
                    _kind: Some(packaged_source_info._kind.clone()),
                    should_translate: Some(packaged_source_info.should_translate),
                    language: Some(crate::utils::conversions::convert_language_string_to_enum(
                        &packaged_source_info.language,
                    )),
                    title: Some(packaged_source_info.title.clone()),
                    email_transform_tag: packaged_source_info.email_transform_tag.clone(),
                    ..Default::default()
                }),
            };
            let create_response = create_source(config, owner, source_name, create_request)?;
            *create_response.source
        }
    };

    if skip_comment_upload {
        return Ok(source);
    }

    // Upload comment batches to the source
    let comment_batch_count = package.get_comment_batch_count();
    for idx in 0..comment_batch_count {
        let batch = package.get_comment_batch(&packaged_source_info.id, CommentBatchKey(idx))?;
        let new_comments: Vec<CommentNew> = batch
            .into_iter()
            .map(|c| {
                let comment = c.comment;
                openapi::models::CommentNew {
                    id: comment.id.clone(),
                    ..Default::default()
                }
            })
            .collect();

        if *resume_on_error {
            // Use cleaner wrapper function for split-on-failure
            let result = sync_comments_with_split_on_failure(
                config,
                owner,
                source_name,
                new_comments.clone(),
                Some(no_charge),
            )?;

            statistics.add_comment_batch_upload(1);
            statistics.add_failed_comment_uploads(result.num_failed);
            handle_split_on_failure_result(result, new_comments.len(), "sync_comments");
        } else {
            // Use add_comments - exact same as original implementation
            let add_request = AddCommentsRequest {
                comments: new_comments.clone(),
            };
            add_comments(config, owner, source_name, add_request, Some(no_charge))?;
            statistics.add_comment_batch_upload(1);
        }
    }
    Ok(source)
}

fn unpack_cm_dataset(
    config: &Configuration,
    mut packaged_dataset: Dataset,
    dataset_creation_timeout: &u64,
    packaged_to_new_source_id: &HashMap<String, String>,
    new_project_name: Option<DatasetOrOwnerName>,
) -> Result<Dataset> {
    if let Some(new_project_name) = new_project_name {
        packaged_dataset.owner = new_project_name.0.clone();
    }

    let entify_def_id_to_name: HashMap<String, String> = packaged_dataset
        .entity_defs
        .clone()
        .into_iter()
        .map(|def| (def.id.clone(), def.name.clone()))
        .collect();

    let dataset_full_name = format!("{}/{}", packaged_dataset.owner, packaged_dataset.name);
    let (owner, dataset_name) = parse_dataset_full_name(&dataset_full_name)?;

    match get_dataset(config, owner, dataset_name) {
        Ok(response) => Ok(*response.dataset),
        Err(_) => {
            let new_dataset = DatasetNew {
                title: Some(packaged_dataset.title.clone()),
                description: Some(packaged_dataset.description.clone()),
                entity_defs: Some(
                    packaged_dataset
                        .entity_defs
                        .clone()
                        .into_iter()
                        .map(|def| EntityDefNew {
                            id: Some(def.id.clone()),
                            color: None,
                            name: def.name.clone(),
                            title: def.title,
                            inherits_from: Box::new(InheritsFrom::new()),
                            trainable: def.trainable,
                            rules: def.rules.map(|rule| {
                                Box::new(EntityRuleSetNewApi {
                                    suppressors: rule.suppressors,
                                    choices: rule.choices.map(|choices_vec| {
                                        choices_vec
                                            .into_iter()
                                            .map(|choice| FieldChoiceNewApi {
                                                id: None,
                                                name: choice.name,
                                                values: choice.values,
                                            })
                                            .collect()
                                    }),
                                    regex: rule.regex,
                                })
                            }),
                            _entity_def_flags: Some(
                                def._entity_def_flags.into_iter().map(|flag| flag).collect(),
                            ),
                            instructions: def.instructions,
                        })
                        .collect::<Vec<EntityDefNew>>(),
                ),
                has_sentiment: Some(packaged_dataset.has_sentiment),
                _dataset_flags: Some(packaged_dataset._dataset_flags.clone()),
                general_fields: Some(
                    packaged_dataset
                        .general_fields
                        .iter()
                        .map(|def| GeneralFieldDefNew {
                            api_name: def.api_name.clone(),
                            field_type_id: Some(def.field_type_id.clone()),
                            field_type_name: entify_def_id_to_name.get(&def.field_type_id).cloned(),
                            title: Some(def.api_name.clone()), // Using api_name as title for now
                        })
                        .collect::<Vec<GeneralFieldDefNew>>(),
                ),
                label_defs: Some(
                    packaged_dataset
                        .label_defs
                        .iter()
                        .map(|def| LabelDefNew {
                            external_id: def.external_id.clone(),
                            instructions: def.instructions.clone().map(|s| s),
                            moon_form: def.moon_form.clone().map(|forms| {
                                forms
                                    .into_iter()
                                    .map(|form| openapi::models::MoonFormFieldDefNew {
                                        field_type_id: Some(Some(form.field_type_id.clone())),
                                        id: Some(Some(form.id.clone())),
                                        kind: form.kind,
                                        name: form.name,
                                        instructions: form.instructions,
                                        rule_set: Some(None),
                                    })
                                    .collect()
                            }),
                            name: def.name.clone(),
                            pretrained: def.pretrained.clone().map(|p| {
                                Box::new(BuiltinLabelDefRequest {
                                    id: p.id.clone(),
                                    name: None, // OpenAPI uses enum instead of string
                                })
                            }),
                            title: def.title.clone().map(|s| s),
                            trainable: None,
                        })
                        .collect::<Vec<LabelDefNew>>(),
                ),
                label_groups: None,
                model_family: Some(packaged_dataset.model_family.to_string()),
                source_ids: Some(
                    packaged_dataset
                        .source_ids
                        .iter()
                        .map(|source_id| -> Result<String> {
                            Ok(packaged_to_new_source_id
                                .get(source_id)
                                .context(format!(
                                    "Could not get new source with id {0}",
                                    source_id
                                ))?
                                .clone())
                        })
                        .try_collect::<String, Vec<String>, anyhow::Error>()?,
                ),
                // Add missing required fields
                entity_kinds: None,
                _label_properties: None,
                debug_config_json: None,
                _timezone: None,
                _preferred_locales: None,
                _model_config: None,
                experimental_async_fork_dataset: None,
                copy_annotations_from: None,
            };

            let create_request = CreateDatasetRequest {
                dataset: Box::new(new_dataset),
            };

            let create_response = create_dataset(config, owner, dataset_name, create_request)
                .context("Could not create dataset")?;
            let dataset = *create_response.dataset;

            wait_for_dataset_to_exist(&dataset, config, *dataset_creation_timeout)
                .context("Could not create dataset")?;

            Ok(dataset)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn unpack_cm_annotations(
    config: &Configuration,
    packaged_source_info: &str,
    old_to_new_source_id: &HashMap<String, String>,
    package: &mut Package,
    statistics: &Arc<CmStatistics>,
    dataset: &Dataset,
    pool: &mut Pool,
    resume_on_error: &bool,
) -> Result<()> {
    let new_source_id = old_to_new_source_id
        .get(packaged_source_info)
        .context(format!(
            "Could not get new source with id {0}",
            &packaged_source_info
        ))?
        .clone();

    // Use get_source_by_id since we have a source ID rather than owner/name
    let source_response = get_source_by_id(config, &new_source_id.to_string())?;
    let source = *source_response.source;

    let comment_batch_count = package.get_comment_batch_count();
    for idx in 0..comment_batch_count {
        let batch = package.get_comment_batch(packaged_source_info, CommentBatchKey(idx))?;
        let mut new_annotations: Vec<NewAnnotation> = batch
            .into_iter()
            .filter_map(|c| {
                if c.has_annotations() {
                    Some(NewAnnotation {
                        comment: CommentIdComment { id: c.comment.id },
                        labelling: Some(
                            c.labelling
                                .into_iter()
                                .map(crate::utils::conversions::comments::convert_labelling_group)
                                .collect(),
                        ),
                        moon_forms: c.moon_forms.map(|forms| {
                            forms
                                .into_iter()
                                .map(crate::utils::conversions::convert_moon_form_group)
                                .collect()
                        }),
                        entities: c
                            .entities
                            .map(|entities| crate::utils::conversions::convert_entities(*entities)),
                    })
                } else {
                    None
                }
            })
            .collect();

        upload_batch_of_annotations(
            &mut new_annotations,
            config,
            &source,
            statistics,
            &DatasetFullName(format!("{}/{}", dataset.owner, dataset.name)),
            pool,
            *resume_on_error,
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct DatasetOrOwnerName(String);

impl DatasetOrOwnerName {
    fn to_dataset_name(&self) -> String {
        self.0.clone()
    }

    fn to_project_name(&self) -> ProjectName {
        ProjectName(self.0.clone())
    }
}

impl FromStr for DatasetOrOwnerName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(DatasetOrOwnerName(s.to_string()))
    }
}

#[allow(clippy::too_many_arguments)]
fn unpack_cm(
    config: &Configuration,
    packaged_dataset: Dataset,
    package: &mut Package,
    dataset_creation_timeout: &u64,
    pool: &mut Pool,
    resume_on_error: &bool,
    no_charge: bool,
    new_project_name: &Option<DatasetOrOwnerName>,
    skip_comment_upload: &bool,
) -> Result<()> {
    let total_comment_batches = package.get_comment_batch_count();
    let total_email_batches = package.get_emails_batch_count();
    let total_batches = total_comment_batches + total_email_batches;

    let statistics = Arc::new(CmStatistics::new());
    let _progress_bar = get_cm_progress_bar(total_batches as u64, &statistics);

    let packaged_sources: Vec<Source> = packaged_dataset
        .source_ids
        .iter()
        .map(|source_id| package.get_source_by_id(source_id))
        .try_collect()?;

    let mut packaged_to_new_source_id: HashMap<String, String> = HashMap::new();

    if let Some(new_project_name) = new_project_name {
        let project_name = new_project_name.to_project_name();
        match get_project(config, &project_name.0) {
            Ok(_) => {}
            Err(_) => {
                // Get current user to use their ID for project creation
                let current_user = get_current_user(config)?;
                let current_user_id = current_user.id;
                create_project_with_wait(
                    config,
                    project_name.0.clone(),
                    None,
                    None,
                    vec![current_user_id],
                )?;
                refresh_user_permissions(config)?;
            }
        }
    }

    for packaged_source_info in packaged_sources {
        // Unpack bucket, if it exists
        let bucket = if let Some(bucket_id) = &packaged_source_info.bucket_id {
            let bucket = unpack_cm_bucket(
                config,
                bucket_id,
                package,
                resume_on_error,
                &statistics,
                no_charge,
                new_project_name.clone(),
            )?;
            Some(bucket)
        } else {
            None
        };

        // Unpack Source
        let source = unpack_cm_source(
            config,
            packaged_source_info.clone(),
            bucket,
            package,
            resume_on_error,
            no_charge,
            &statistics,
            new_project_name.clone(),
            *skip_comment_upload,
        )?;

        packaged_to_new_source_id.insert(packaged_source_info.id.clone(), source.id.clone());
    }

    // Unpack Dataset
    let dataset = unpack_cm_dataset(
        config,
        packaged_dataset.clone(),
        dataset_creation_timeout,
        &packaged_to_new_source_id,
        new_project_name.clone(),
    )?;

    // Upload annotations
    for package_source_id in &packaged_dataset.source_ids {
        unpack_cm_annotations(
            config,
            package_source_id,
            &packaged_to_new_source_id,
            package,
            &statistics,
            &dataset,
            pool,
            resume_on_error,
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn unpack_ixp(
    dataset: Box<Dataset>,
    package: &mut Package,
    new_project_name: Option<DatasetOrOwnerName>,
    dataset_creation_timeout: &u64,
    pool: &mut Pool,
    resume_on_error: &bool,
    max_attachment_memory_mb: &u64,
    config: &Configuration,
) -> Result<()> {
    let total_documents = package.get_document_count();
    let statistics = Arc::new(IxpStatistics::new());
    let _progress_bar = get_ixp_progress_bar(total_documents as u64, &statistics);

    let packaged_sources = get_ixp_source(&dataset, SourceProvider::Packaged(package))
        .context("Could not get ixp source from package")?;

    // We use title here as the name will already have a hex appended, the api will normalize
    // the title into an api name
    let new_dataset = create_ixp_dataset(
        new_project_name
            .map(|p| p.to_dataset_name())
            .clone()
            .unwrap_or(dataset.title.clone()),
        dataset.label_defs.clone(),
        config,
        *dataset_creation_timeout,
        dataset._model_config.as_ref().clone(),
        dataset
            .entity_defs
            .clone()
            .into_iter()
            .map(|def| EntityDefNew {
                id: Some(def.id),
                color: None,
                name: def.name,
                title: def.title,
                inherits_from: Box::new(InheritsFrom::new()),
                trainable: def.trainable,
                rules: def.rules.map(|rule| {
                    Box::new(EntityRuleSetNewApi {
                        suppressors: rule.suppressors,
                        choices: rule.choices.map(|choices_vec| {
                            choices_vec
                                .into_iter()
                                .map(|choice| FieldChoiceNewApi {
                                    id: None,
                                    name: choice.name,
                                    values: choice.values,
                                })
                                .collect()
                        }),
                        regex: rule.regex,
                    })
                }),
                _entity_def_flags: Some(
                    def._entity_def_flags.into_iter().map(|flag| flag).collect(),
                ),
                instructions: def.instructions,
            })
            .collect(),
    )
    .context("Could not create dataset")?;

    let new_sources = get_ixp_source(&new_dataset, SourceProvider::Remote(config))
        .context("Could not get ixp source via api")?;

    for (packaged, new) in [
        (&packaged_sources.design_time, &new_sources.design_time),
        (&packaged_sources.runtime, &new_sources.runtime),
    ] {
        unpack_ixp_source(
            packaged,
            new,
            &new_dataset,
            package,
            config,
            pool,
            *resume_on_error,
            &statistics,
            max_attachment_memory_mb,
        )
        .context("Failed to unpack ixp source")?;
    }
    Ok(())
}

pub fn run(args: &UploadPackageArgs, config: &Configuration, pool: &mut Pool) -> Result<()> {
    refresh_user_permissions(config)?;
    let UploadPackageArgs {
        file,
        resume_on_error,
        max_attachment_memory_mb,
        dataset_creation_timeout,
        new_project_name,
        no_charge,
        yes,
        skip_comment_upload,
    } = args;

    let mut has_consented_to_ai_unit_consumption = false;

    let mut package =
        Package::new(file).context("Failed to read package, check path is correct.")?;

    for dataset in package
        .datasets()
        .context("Could not get package datasets")?
    {
        if dataset._dataset_flags.contains(&DatasetFlag::Ixp) {
            unpack_ixp(
                Box::new(dataset),
                &mut package,
                new_project_name.clone(),
                dataset_creation_timeout,
                pool,
                resume_on_error,
                max_attachment_memory_mb,
                config,
            )?;
        } else {
            if !no_charge && !yes && !has_consented_to_ai_unit_consumption {
                ensure_uip_user_consents_to_ai_unit_charge(&config.base_path.parse()?)?;
                has_consented_to_ai_unit_consumption = true;
            }
            // Attempt to unpack the dataset with retries for project creation due to race
            // condition
            for attempt in 1..=MAX_UNPACK_CM_ATTEMPTS {
                match unpack_cm(
                    config,
                    dataset.clone(),
                    &mut package,
                    dataset_creation_timeout,
                    pool,
                    resume_on_error,
                    *no_charge,
                    new_project_name,
                    skip_comment_upload,
                ) {
                    Ok(_) => break,
                    Err(err) if is_project_not_found_error(&err) && new_project_name.is_some() => {
                        sleep(Duration::from_secs(UNPACK_CM_SLEEP_SECONDS));
                        refresh_user_permissions(config)?;

                        if attempt == MAX_UNPACK_CM_ATTEMPTS {
                            return Err(anyhow!(
                                "Failed to upload package after {} attempts. Project creation timed out. Last error: {}",
                                MAX_UNPACK_CM_ATTEMPTS,
                                err
                            ));
                        }
                    }
                    Err(err) => return Err(err.context("Failed to unpack dataset")),
                }
            }
        }
    }
    log::info!("Package uploaded.");
    Ok(())
}

fn is_project_not_found_error(err: &anyhow::Error) -> bool {
    err.to_string().contains("No such project")
}

#[derive(Debug)]
pub struct CmStatistics {
    comment_uploads: AtomicUsize,
    email_uploads: AtomicUsize,
    annotations: AtomicUsize,
    failed_comment_uploads: AtomicUsize,
    failed_email_uploads: AtomicUsize,
    failed_annotations: AtomicUsize,
}

impl AnnotationStatistic for Arc<CmStatistics> {
    fn add_annotation(&self) {
        self.annotations.fetch_add(1, Ordering::SeqCst);
    }

    fn add_failed_annotation(&self) {
        self.failed_annotations.fetch_add(1, Ordering::SeqCst);
    }
}

impl CmStatistics {
    fn new() -> Self {
        Self {
            comment_uploads: AtomicUsize::new(0),
            email_uploads: AtomicUsize::new(0),
            annotations: AtomicUsize::new(0),
            failed_annotations: AtomicUsize::new(0),
            failed_email_uploads: AtomicUsize::new(0),
            failed_comment_uploads: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_comment_batch_upload(&self, num: usize) {
        self.comment_uploads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_email_batch_uploads(&self, num: usize) {
        self.email_uploads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_failed_comment_uploads(&self, num: usize) {
        self.failed_comment_uploads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_failed_email_uploads(&self, num: usize) {
        self.failed_email_uploads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn num_comment_batches_uploaded(&self) -> usize {
        self.comment_uploads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_email_batches_uploaded(&self) -> usize {
        self.email_uploads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_annotations(&self) -> usize {
        self.annotations.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_comment_uploads(&self) -> usize {
        self.failed_comment_uploads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_email_uploads(&self) -> usize {
        self.failed_email_uploads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_annotations(&self) -> usize {
        self.failed_annotations.load(Ordering::SeqCst)
    }
}

fn get_cm_progress_bar(total: u64, statistics: &Arc<CmStatistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let num_comments_uploaded = statistics.num_comment_batches_uploaded();
            let num_emails_uploaded = statistics.num_email_batches_uploaded();
            let num_annotations = statistics.num_annotations();

            let num_failed_comments = statistics.num_failed_comment_uploads();
            let num_failed_emails = statistics.num_failed_email_uploads();
            let num_failed_annotations = statistics.num_failed_annotations();

            fn get_failed_part(val: usize, name: &str) -> String {
                if val > 0 {
                    format!(" {} {}", val.to_string().bold(), name.dimmed())
                } else {
                    String::new()
                }
            }

            (
                (num_comments_uploaded + num_emails_uploaded) as u64,
                format!(
                    "{} {} {} {} {} {}{}{}{}",
                    num_emails_uploaded.to_string().bold(),
                    "email batches".dimmed(),
                    num_comments_uploaded.to_string().bold(),
                    "comment batches".dimmed(),
                    num_annotations.to_string().bold(),
                    "annotations".dimmed(),
                    get_failed_part(num_failed_comments, "failed comments"),
                    get_failed_part(num_failed_emails, "failed emails"),
                    get_failed_part(num_failed_annotations, "failed annotations")
                ),
            )
        },
        statistics,
        Some(total),
        crate::progress::Options { bytes_units: false },
    )
}

#[derive(Debug)]
pub struct IxpStatistics {
    document_reads: AtomicUsize,
    document_uploads: AtomicUsize,
    annotations: AtomicUsize,
    failed_annotations: AtomicUsize,
    failed_reads: AtomicUsize,
    failed_uploads: AtomicUsize,
}

impl IxpStatistics {
    fn new() -> Self {
        Self {
            annotations: AtomicUsize::new(0),
            document_uploads: AtomicUsize::new(0),
            document_reads: AtomicUsize::new(0),
            failed_annotations: AtomicUsize::new(0),
            failed_reads: AtomicUsize::new(0),
            failed_uploads: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_annotation(&self, num: usize) {
        self.annotations.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_failed_annotations(&self, num: usize) {
        self.failed_annotations.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_failed_uploads(&self, num: usize) {
        self.failed_uploads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_document_uploads(&self, num: usize) {
        self.document_uploads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn add_document_reads(&self, num: usize) {
        self.document_reads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn num_document_uploads(&self) -> usize {
        self.document_uploads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_document_reads(&self) -> usize {
        self.document_reads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_annotations(&self) -> usize {
        self.annotations.load(Ordering::SeqCst)
    }

    #[inline]
    fn add_failed_document_reads(&self, num: usize) {
        self.failed_reads.fetch_add(num, Ordering::SeqCst);
    }

    #[inline]
    fn num_failed_annotations(&self) -> usize {
        self.failed_annotations.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_reads(&self) -> usize {
        self.failed_reads.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_uploads(&self) -> usize {
        self.failed_uploads.load(Ordering::SeqCst)
    }
}

fn get_ixp_progress_bar(total_comments: u64, statistics: &Arc<IxpStatistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let num_docs_read = statistics.num_document_reads();
            let num_docs_uploaded = statistics.num_document_uploads();
            let num_annotations = statistics.num_annotations();
            let num_failed_annotations = statistics.num_failed_annotations();
            let num_failed_reads = statistics.num_failed_reads();
            let num_failed_uploads = statistics.num_failed_uploads();

            fn get_failed_part(val: usize, name: &str) -> String {
                if val > 0 {
                    format!(" {} {}", val.to_string().bold(), name.dimmed())
                } else {
                    String::new()
                }
            }

            (
                ((num_docs_uploaded + num_docs_read) / 2) as u64,
                format!(
                    "{} {} {} {} {} {}{}{}{}",
                    num_docs_read.to_string().bold(),
                    "docs read".dimmed(),
                    num_docs_uploaded.to_string().bold(),
                    "docs uploaded".dimmed(),
                    num_annotations.to_string().bold(),
                    "annotations".dimmed(),
                    get_failed_part(num_failed_reads, "failed reads"),
                    get_failed_part(num_failed_uploads, "failed uploads"),
                    get_failed_part(num_failed_annotations, "failed annotations")
                ),
            )
        },
        statistics,
        Some(total_comments),
        crate::progress::Options { bytes_units: false },
    )
}
