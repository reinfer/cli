use colored::Colorize;
use itertools::Itertools;
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
use reinfer_client::{
    resources::{
        bucket::{Bucket, Id as BucketId},
        dataset::{DatasetFlag, IxpDatasetNew, ModelConfig},
        entity_def::{EntityRuleSetNew, FieldChoiceNew, Name as EntityDefName, NewGeneralFieldDef},
    },
    Client, CommentUid, Dataset, DatasetFullName, DatasetName, EntityDefId, LabelDef,
    NewAnnotatedComment, NewBucket, NewComment, NewDataset, NewEntityDef, NewLabelDef,
    NewLabelDefPretrained, NewSource, ProjectName, Source, SourceId, SourceKind, UpdateDataset,
    Username, DEFAULT_LABEL_GROUP_NAME,
};
use scoped_threadpool::Pool;
use structopt::StructOpt;

use crate::{
    commands::{
        auth::refresh_user_permissions,
        create::{
            annotations::{
                upload_batch_of_annotations, AnnotationStatistic, CommentIdComment, NewAnnotation,
            },
            project::create_project,
        },
        ensure_uip_user_consents_to_ai_unit_charge,
        package::{AttachmentKey, CommentBatchKey, EmailBatchKey, Package},
    },
    progress::Progress,
};

const MAX_UNPACK_CM_ATTEMPTS: u64 = 20;
const UNPACK_CM_SLEEP_SECONDS: u64 = 3;

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
}

fn wait_for_dataset_to_exist(dataset: &Dataset, client: &Client, timeout_s: u64) -> Result<()> {
    let start_time = Instant::now();

    while (start_time - Instant::now()).as_secs() <= timeout_s {
        refresh_user_permissions(client, false)?;
        let datasets = client.get_datasets()?;

        let dataset_exists = datasets
            .iter()
            .map(|dataset| dataset.full_name())
            .contains(&dataset.full_name());

        if dataset_exists {
            return Ok(());
        } else {
            sleep(Duration::from_millis(500));
        }
    }

    Err(anyhow!("Timeout waiting for dataset to be created"))
}

fn create_ixp_dataset(
    name: DatasetName,
    label_defs: Vec<LabelDef>,
    client: &Client,
    timeout_s: u64,
    model_config: ModelConfig,
    entity_defs: Vec<NewEntityDef>,
) -> Result<Dataset> {
    let mut new_label_defs = Vec::new();

    label_defs.iter().try_for_each(|label| -> Result<()> {
        new_label_defs.push(NewLabelDef::try_from(label)?);
        Ok(())
    })?;

    let dataset = client.create_ixp_dataset(IxpDatasetNew { name })?;

    wait_for_dataset_to_exist(&dataset, client, timeout_s)?;

    client.update_dataset(
        &dataset.full_name(),
        UpdateDataset {
            model_config: Some(model_config),
            source_ids: None,
            title: None,
            description: None,
            entity_defs,
        },
    )?;

    client
        .create_label_defs_bulk(
            &dataset.full_name(),
            DEFAULT_LABEL_GROUP_NAME.clone(),
            new_label_defs,
        )
        .context("Error when creating label defs")?;
    Ok(dataset)
}

pub struct IxpDatasetSources {
    runtime: Source,
    design_time: Source,
}

enum SourceProvider<'a> {
    Packaged(&'a mut Package),
    Remote(&'a Client),
}

impl SourceProvider<'_> {
    fn get_source(&mut self, source_id: &SourceId) -> Result<Source> {
        match self {
            Self::Packaged(package) => package.get_source_by_id(source_id),
            Self::Remote(client) => client
                .get_source(source_id.clone())
                .map_err(anyhow::Error::msg),
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
                .get_source(source_id)
                .context("Could not get source")?;
            if source_by_kind.contains_key(&source.kind) {
                return Err(anyhow!("Duplicate source kind found in packaged dataset"));
            }

            source_by_kind.insert(source.kind.clone(), source.clone());
            Ok(())
        })?;

    let runtime = source_by_kind
        .get(&SourceKind::IxpRuntime)
        .context(anyhow!(
            "Could not find runtime source for {0} dataset {1}",
            provider,
            dataset.id.0
        ))?
        .clone();

    let design_time = source_by_kind
        .get(&SourceKind::IxpDesignTime)
        .context(anyhow!(
            "Could not find design time source for {0} dataset {1}",
            provider,
            dataset.id.0
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
    client: &Client,
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
                                comment: comment.clone(),
                                content: result,
                                size: attachment.size as usize,
                                file_name: attachment.name.clone(),
                            });

                            if should_upload_batch(&documents, max_attachment_memory_mb) {
                                upload_batch(
                                    &documents,
                                    client,
                                    &new_source.id,
                                    &new_dataset.full_name(),
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
        client,
        &new_source.id,
        &new_dataset.full_name(),
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
    client: &Client,
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
                let upload_result = client
                    .upload_ixp_document(source_id, item.file_name.clone(), item.content.clone())
                    .map_err(anyhow::Error::msg);

                match upload_result {
                    Ok(new_comment_id) => {
                        statistics.add_document_uploads(1);

                        let comment_uid =
                            CommentUid(format!("{}.{}", source_id.0, new_comment_id.0));

                        if item.comment.has_annotations() {
                            let labelling_result = client.update_labelling(
                                dataset_name,
                                &comment_uid,
                                None,
                                item.comment.entities.as_ref(),
                                item.comment.moon_forms.as_deref(),
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
    client: &Client,
    packaged_bucket_id: &BucketId,
    package: &mut Package,
    resume_on_error: &bool,
    statistics: &Arc<CmStatistics>,
    no_charge: bool,
    new_project_name: Option<DatasetOrOwnerName>,
) -> Result<Bucket> {
    let mut packaged_bucket = package.get_bucket_by_id(packaged_bucket_id)?;

    if let Some(new_project_name) = new_project_name {
        packaged_bucket.owner = new_project_name.to_username();
    }

    // Get Existing bucket, or create if it doesn't exist
    let bucket = match client.get_bucket(packaged_bucket.full_name()) {
        Ok(bucket) => bucket,
        Err(_) => client.create_bucket(
            &packaged_bucket.full_name(),
            NewBucket {
                bucket_type: reinfer_client::BucketType::Emails,
                title: None,
            },
        )?,
    };

    // Upload batches of emails to the bucket
    let email_batch_count = package.get_email_batch_count_for_bucket(&packaged_bucket.id);
    for idx in 0..email_batch_count {
        let batch = package
            .get_email_batch(&packaged_bucket.id, EmailBatchKey(idx))
            .context("Could not get email batch")?;

        if *resume_on_error {
            let result =
                client.put_emails_split_on_failure(&bucket.full_name(), batch, no_charge)?;

            statistics.add_email_batch_uploads(1);
            statistics.add_failed_email_uploads(result.num_failed);
        } else {
            client.put_emails(&bucket.full_name(), batch, no_charge)?;
            statistics.add_email_batch_uploads(1);
        }
    }
    Ok(bucket)
}

#[allow(clippy::too_many_arguments)]
fn unpack_cm_source(
    client: &Client,
    mut packaged_source_info: Source,
    bucket: Option<Bucket>,
    package: &mut Package,
    resume_on_error: &bool,
    no_charge: bool,
    statistics: &Arc<CmStatistics>,
    new_project_name: Option<DatasetOrOwnerName>,
) -> Result<Source> {
    // Get existing source or create a new one if it doesn't exist
    if let Some(project_name) = new_project_name {
        packaged_source_info.owner = project_name.to_username();
    }

    let source = match client.get_source(packaged_source_info.full_name()) {
        Ok(source) => source,
        Err(_) => client.create_source(
            &packaged_source_info.full_name(),
            NewSource {
                bucket_id: bucket.map(|bucket| bucket.id),
                description: Some(&packaged_source_info.description),
                kind: Some(&packaged_source_info.kind),
                should_translate: Some(packaged_source_info.should_translate),
                language: Some(&packaged_source_info.language),
                title: Some(&packaged_source_info.title),
                transform_tag: packaged_source_info.transform_tag.as_ref(),
                ..Default::default()
            },
        )?,
    };
    // Upload comment batches to the source
    let comment_batch_count = package.get_comment_batch_count();
    for idx in 0..comment_batch_count {
        let batch = package.get_comment_batch(&packaged_source_info.id, CommentBatchKey(idx))?;
        let new_comments: Vec<NewComment> = batch.into_iter().map(|c| c.comment).collect();

        if *resume_on_error {
            let result = client.sync_comments_split_on_failure(
                &source.full_name(),
                new_comments,
                no_charge,
            )?;

            statistics.add_comment_batch_upload(1);
            statistics.add_failed_comment_uploads(result.num_failed);
        } else {
            client.sync_comments(&source.full_name(), new_comments, no_charge)?;
            statistics.add_comment_batch_upload(1);
        }
    }
    Ok(source)
}

fn unpack_cm_dataset(
    client: &Client,
    mut packaged_dataset: Dataset,
    dataset_creation_timeout: &u64,
    packaged_to_new_source_id: &HashMap<SourceId, SourceId>,
    new_project_name: Option<DatasetOrOwnerName>,
) -> Result<Dataset> {
    if let Some(new_project_name) = new_project_name {
        packaged_dataset.owner = new_project_name.to_username();
    }

    let entify_def_id_to_name: HashMap<EntityDefId, EntityDefName> = packaged_dataset
        .entity_defs
        .clone()
        .into_iter()
        .map(|def| (def.id, def.name))
        .collect();

    match client.get_dataset(packaged_dataset.full_name()) {
        Ok(dataset) => Ok(dataset),
        Err(_) => {
            let dataset = client
                .create_dataset(
                    &packaged_dataset.full_name(),
                    NewDataset {
                        description: Some(&packaged_dataset.description),
                        entity_defs: Some(
                            &packaged_dataset
                                .entity_defs
                                .clone()
                                .into_iter()
                                .map(|def| NewEntityDef {
                                    entity_def_flags: def.entity_def_flags,
                                    inherits_from: def.inherits_from,
                                    name: def.name,
                                    rules: def.rules.map(|rule| EntityRuleSetNew {
                                        suppressors: rule.suppressors,
                                        choices: rule
                                            .choices
                                            .into_iter()
                                            .map(|choice| FieldChoiceNew {
                                                name: choice.name,
                                                values: choice.values,
                                            })
                                            .collect(),
                                        regex: rule.regex,
                                    }),

                                    title: def.title,
                                    trainable: def.trainable,
                                    instructions: def.instructions,
                                })
                                .collect::<Vec<NewEntityDef>>(),
                        ),
                        has_sentiment: Some(packaged_dataset.has_sentiment),
                        dataset_flags: packaged_dataset.clone().dataset_flags,
                        copy_annotations_from: None,
                        general_fields: Some(
                            &packaged_dataset
                                .general_fields
                                .iter()
                                .map(|def| NewGeneralFieldDef {
                                    api_name: def.api_name.clone(),
                                    field_type_id: def.field_type_id.clone(),
                                    field_type_name: if let Some(field_type_id) = &def.field_type_id
                                    {
                                        entify_def_id_to_name
                                            .get(&EntityDefId(field_type_id.clone()))
                                            .cloned()
                                    } else {
                                        None
                                    },
                                })
                                .collect::<Vec<NewGeneralFieldDef>>(),
                        ),
                        label_defs: Some(
                            &packaged_dataset
                                .label_defs
                                .iter()
                                .map(|def| NewLabelDef {
                                    external_id: def.external_id.clone(),
                                    instructions: Some(def.instructions.clone()),
                                    moon_form: def.moon_form.clone(),
                                    name: def.name.clone(),
                                    pretrained: def.pretrained.clone().map(|p| {
                                        NewLabelDefPretrained {
                                            id: p.id.clone(),
                                            name: Some(p.name.clone()),
                                        }
                                    }),
                                    title: Some(def.title.clone()),
                                    trainable: None,
                                })
                                .collect::<Vec<NewLabelDef>>(),
                        ),
                        label_groups: None,
                        model_family: Some(&packaged_dataset.model_family.0),
                        source_ids: &packaged_dataset
                            .source_ids
                            .iter()
                            .map(|source_id| -> Result<SourceId> {
                                Ok(packaged_to_new_source_id
                                    .get(source_id)
                                    .context(format!(
                                        "Could not get new source with id {0}",
                                        source_id.0
                                    ))?
                                    .clone())
                            })
                            .try_collect::<SourceId, Vec<SourceId>, anyhow::Error>()?,
                        title: Some(&packaged_dataset.title),
                    },
                )
                .context("Could not create dataset")?;

            wait_for_dataset_to_exist(&dataset, client, *dataset_creation_timeout)
                .context("Could not create dataset")?;

            Ok(dataset)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn unpack_cm_annotations(
    client: &Client,
    packaged_source_info: &SourceId,
    old_to_new_source_id: &HashMap<SourceId, SourceId>,
    package: &mut Package,
    statistics: &Arc<CmStatistics>,
    dataset: &Dataset,
    pool: &mut Pool,
    resume_on_error: &bool,
) -> Result<()> {
    let source = client.get_source(
        old_to_new_source_id
            .get(packaged_source_info)
            .context(format!(
                "Could not get new source with id {0}",
                packaged_source_info.0
            ))?
            .clone(),
    )?;

    let comment_batch_count = package.get_comment_batch_count();
    for idx in 0..comment_batch_count {
        let batch = package.get_comment_batch(packaged_source_info, CommentBatchKey(idx))?;
        let mut new_annotations: Vec<NewAnnotation> = batch
            .into_iter()
            .filter_map(|c| {
                if c.has_annotations() {
                    Some(NewAnnotation {
                        comment: CommentIdComment { id: c.comment.id },
                        labelling: c.labelling,
                        moon_forms: c.moon_forms,
                        entities: c.entities,
                    })
                } else {
                    None
                }
            })
            .collect();

        upload_batch_of_annotations(
            &mut new_annotations,
            client,
            &source,
            statistics,
            &dataset.full_name(),
            pool,
            *resume_on_error,
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct DatasetOrOwnerName(String);

impl DatasetOrOwnerName {
    fn to_dataset_name(&self) -> DatasetName {
        DatasetName(self.0.clone())
    }

    fn to_username(&self) -> Username {
        Username(self.0.clone())
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
    client: &Client,
    packaged_dataset: Dataset,
    package: &mut Package,
    dataset_creation_timeout: &u64,
    pool: &mut Pool,
    resume_on_error: &bool,
    no_charge: bool,
    new_project_name: &Option<DatasetOrOwnerName>,
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

    let mut packaged_to_new_source_id: HashMap<SourceId, SourceId> = HashMap::new();

    if let Some(new_project_name) = new_project_name {
        match client.get_project(&new_project_name.to_project_name()) {
            Ok(_) => {}
            Err(_) => {
                create_project(
                    client,
                    &new_project_name.to_project_name(),
                    &None,
                    &None,
                    &[client.get_current_user()?.id],
                )?;
                refresh_user_permissions(client, false)?;
            }
        }
    }

    for packaged_source_info in packaged_sources {
        // Unpack bucket, if it exists
        let bucket = if let Some(bucket_id) = &packaged_source_info.bucket_id {
            let bucket = unpack_cm_bucket(
                client,
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
            client,
            packaged_source_info.clone(),
            bucket,
            package,
            resume_on_error,
            no_charge,
            &statistics,
            new_project_name.clone(),
        )?;

        packaged_to_new_source_id.insert(packaged_source_info.id.clone(), source.id.clone());
    }

    // Unpack Dataset
    let dataset = unpack_cm_dataset(
        client,
        packaged_dataset.clone(),
        dataset_creation_timeout,
        &packaged_to_new_source_id,
        new_project_name.clone(),
    )?;

    // Upload annotations
    for package_source_id in &packaged_dataset.source_ids {
        unpack_cm_annotations(
            client,
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
    client: &Client,
    dataset: Dataset,
    package: &mut Package,
    new_project_name: Option<DatasetOrOwnerName>,
    dataset_creation_timeout: &u64,
    pool: &mut Pool,
    resume_on_error: &bool,
    max_attachment_memory_mb: &u64,
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
            .unwrap_or(DatasetName(dataset.title)),
        dataset.label_defs,
        client,
        *dataset_creation_timeout,
        dataset.model_config,
        dataset
            .entity_defs
            .into_iter()
            .map(|def| NewEntityDef {
                entity_def_flags: def.entity_def_flags,
                inherits_from: def.inherits_from,
                name: def.name,
                rules: def.rules.map(|rule| EntityRuleSetNew {
                    suppressors: rule.suppressors,
                    choices: rule
                        .choices
                        .into_iter()
                        .map(|choice| FieldChoiceNew {
                            name: choice.name,
                            values: choice.values,
                        })
                        .collect(),
                    regex: rule.regex,
                }),
                title: def.title,
                trainable: def.trainable,
                instructions: def.instructions,
            })
            .collect(),
    )
    .context("Could not create dataset")?;

    let new_sources = get_ixp_source(&new_dataset, SourceProvider::Remote(client))
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
            client,
            pool,
            *resume_on_error,
            &statistics,
            max_attachment_memory_mb,
        )
        .context("Failed to unpack ixp source")?;
    }
    Ok(())
}

pub fn run(args: &UploadPackageArgs, client: &Client, pool: &mut Pool) -> Result<()> {
    refresh_user_permissions(client, false)?;
    let UploadPackageArgs {
        file,
        resume_on_error,
        max_attachment_memory_mb,
        dataset_creation_timeout,
        new_project_name,
        no_charge,
        yes,
    } = args;

    let mut has_consented_to_ai_unit_consumption = false;

    let mut package =
        Package::new(file).context("Failed to read package, check path is correct.")?;

    for dataset in package
        .datasets()
        .context("Could not get package datasets")?
    {
        if dataset.has_flag(DatasetFlag::Ixp) {
            unpack_ixp(
                client,
                dataset,
                &mut package,
                new_project_name.clone(),
                dataset_creation_timeout,
                pool,
                resume_on_error,
                max_attachment_memory_mb,
            )?;
        } else {
            if !no_charge && !yes && !has_consented_to_ai_unit_consumption {
                ensure_uip_user_consents_to_ai_unit_charge(client.base_url())?;
                has_consented_to_ai_unit_consumption = true;
            }
            // Attempt to unpack the dataset with retries for project creation due to race
            // condition
            for attempt in 1..=MAX_UNPACK_CM_ATTEMPTS {
                match unpack_cm(
                    client,
                    dataset.clone(),
                    &mut package,
                    dataset_creation_timeout,
                    pool,
                    resume_on_error,
                    *no_charge,
                    new_project_name,
                ) {
                    Ok(_) => break,
                    Err(err) if is_project_not_found_error(&err) && new_project_name.is_some() => {
                        sleep(Duration::from_secs(UNPACK_CM_SLEEP_SECONDS));
                        refresh_user_permissions(client, false)?;

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
