#![deny(clippy::all)]
mod error;
pub mod resources;
pub mod retry;

use chrono::{DateTime, Utc};
use futures::Stream;
use once_cell::sync::Lazy;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client as HttpClient, IntoUrl, Response as HttpResponse, Result as ReqwestResult,
};
use resources::project::ForceDeleteProject;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{cell::Cell, fmt::Display, future::Future};
use url::Url;

use crate::resources::{
    bucket::{
        CreateRequest as CreateBucketRequest, CreateResponse as CreateBucketResponse,
        GetAvailableResponse as GetAvailableBucketsResponse, GetResponse as GetBucketResponse,
    },
    comment::{
        GetAnnotationsResponse, GetCommentResponse, GetLabellingsAfter, GetPredictionsResponse,
        GetRecentRequest, PutCommentsRequest, PutCommentsResponse, RecentCommentsPage,
        SyncCommentsRequest, UpdateAnnotationsRequest,
    },
    dataset::{
        CreateRequest as CreateDatasetRequest, CreateResponse as CreateDatasetResponse,
        GetAvailableResponse as GetAvailableDatasetsResponse, GetResponse as GetDatasetResponse,
        UpdateRequest as UpdateDatasetRequest, UpdateResponse as UpdateDatasetResponse,
    },
    email::{PutEmailsRequest, PutEmailsResponse},
    project::{
        CreateProjectRequest, CreateProjectResponse, GetProjectResponse, GetProjectsResponse,
        UpdateProjectRequest, UpdateProjectResponse,
    },
    source::{
        CreateRequest as CreateSourceRequest, CreateResponse as CreateSourceResponse,
        GetAvailableResponse as GetAvailableSourcesResponse, GetResponse as GetSourceResponse,
        UpdateRequest as UpdateSourceRequest, UpdateResponse as UpdateSourceResponse,
    },
    statistics::GetResponse as GetStatisticsResponse,
    trigger::{
        AdvanceRequest as TriggerAdvanceRequest, FetchRequest as TriggerFetchRequest,
        GetResponse as GetTriggersResponse, ResetRequest as TriggerResetRequest,
        TagExceptionsRequest as TagTriggerExceptionsRequest,
    },
    user::GetResponse as GetUserResponse,
    user::{
        CreateRequest as CreateUserRequest, CreateResponse as CreateUserResponse,
        GetAvailableResponse as GetAvailableUsersResponse,
        GetCurrentResponse as GetCurrentUserResponse, PostUserRequest, PostUserResponse,
        WelcomeEmailResponse,
    },
    EmptySuccess, Response,
};

use crate::retry::{Retrier, RetryConfig};

pub use crate::{
    error::{Error, Result},
    resources::{
        bucket::{
            Bucket, BucketType, FullName as BucketFullName, Id as BucketId,
            Identifier as BucketIdentifier, Name as BucketName, NewBucket, TransformTag,
        },
        comment::{
            AnnotatedComment, Comment, CommentFilter, CommentsIterPage, Continuation,
            EitherLabelling, Entities, Entity, HasAnnotations, Id as CommentId, Label, Labelling,
            Message, MessageBody, MessageSignature, MessageSubject, NewAnnotatedComment,
            NewComment, NewEntities, NewLabelling, NewMoonForm, PredictedLabel, Prediction,
            PropertyMap, PropertyValue, Sentiment, SyncCommentsResponse, Uid as CommentUid,
        },
        dataset::{
            Dataset, FullName as DatasetFullName, Id as DatasetId, Identifier as DatasetIdentifier,
            ModelVersion, Name as DatasetName, NewDataset, UpdateDataset,
        },
        email::{Id as EmailId, Mailbox, MimeContent, NewEmail},
        entity_def::{EntityDef, Id as EntityDefId, Name as EntityName, NewEntityDef},
        label_def::{
            LabelDef, LabelDefPretrained, Name as LabelName, NewLabelDef, NewLabelDefPretrained,
            PretrainedId as LabelDefPretrainedId,
        },
        label_group::{
            LabelGroup, Name as LabelGroupName, NewLabelGroup, DEFAULT_LABEL_GROUP_NAME,
        },
        project::{NewProject, Project, ProjectName, UpdateProject},
        source::{
            FullName as SourceFullName, Id as SourceId, Identifier as SourceIdentifier,
            Name as SourceName, NewSource, Source, SourceKind, UpdateSource,
        },
        statistics::Statistics,
        trigger::{
            Batch as TriggerBatch, FullName as TriggerFullName, SequenceId as TriggerSequenceId,
            Trigger, TriggerException, TriggerExceptionMetadata,
        },
        user::{
            Email, GlobalPermission, Id as UserId, Identifier as UserIdentifier,
            ModifiedPermissions, NewUser, ProjectPermission, UpdateUser, User, Username,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token(pub String);

pub struct Config {
    pub endpoint: Url,
    pub token: Token,
    pub accept_invalid_certificates: bool,
    #[cfg(feature = "native")]
    pub proxy: Option<Url>,
    /// Retry settings to use, if any. This will apply to all requests except for POST requests
    /// which are not idempotent (as they cannot be naively retried).
    pub retry_config: Option<RetryConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            endpoint: DEFAULT_ENDPOINT.clone(),
            token: Token("".to_owned()),
            accept_invalid_certificates: false,
            #[cfg(feature = "native")]
            proxy: None,
            retry_config: None,
        }
    }
}

#[derive(Debug)]
pub struct Client {
    endpoints: Endpoints,
    http_client: HttpClient,
    headers: HeaderMap,
    retrier: Option<Retrier>,
}

#[derive(Serialize)]
pub struct GetLabellingsInBulk<'a> {
    pub source_id: &'a SourceId,
    pub return_predictions: &'a bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: &'a Option<GetLabellingsAfter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: &'a Option<usize>,
}

#[derive(Serialize)]
pub struct GetCommentsIterPageQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_timestamp: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_timestamp: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<&'a Continuation>,
    pub limit: usize,
}

impl Client {
    /// Create a new API client.
    pub fn new(config: Config) -> Result<Client> {
        let http_client = {
            let builder = HttpClient::builder();
            #[cfg(feature = "native")]
            let builder = {
                let mut builder = builder
                    .gzip(true)
                    .danger_accept_invalid_certs(config.accept_invalid_certificates);
                if let Some(proxy) = config.proxy.clone() {
                    builder =
                        builder.proxy(reqwest::Proxy::all(proxy).map_err(Error::BuildHttpClient)?);
                }
                builder
            };
            builder.build().map_err(Error::BuildHttpClient)?
        };

        let headers = build_headers(&config)?;
        let endpoints = Endpoints::new(config.endpoint)?;
        let retrier = config.retry_config.map(Retrier::new);
        Ok(Client {
            endpoints,
            http_client,
            headers,
            retrier,
        })
    }

    /// List all visible sources.
    pub async fn get_sources(&self) -> Result<Vec<Source>> {
        Ok(self
            .get::<_, GetAvailableSourcesResponse>(self.endpoints.sources.clone())
            .await?
            .sources)
    }

    /// Get a source by either id or name.
    pub async fn get_user(&self, user: impl Into<UserIdentifier>) -> Result<User> {
        Ok(match user.into() {
            UserIdentifier::Id(user_id) => {
                self.get::<_, GetUserResponse>(self.endpoints.user_by_id(&user_id)?)
                    .await?
                    .user
            }
        })
    }

    /// Get a source by either id or name.
    pub async fn get_source(&self, source: impl Into<SourceIdentifier>) -> Result<Source> {
        Ok(match source.into() {
            SourceIdentifier::Id(source_id) => {
                self.get::<_, GetSourceResponse>(self.endpoints.source_by_id(&source_id)?)
                    .await?
                    .source
            }
            SourceIdentifier::FullName(source_name) => {
                self.get::<_, GetSourceResponse>(self.endpoints.source_by_name(&source_name)?)
                    .await?
                    .source
            }
        })
    }

    /// Create a new source.
    pub async fn create_source(
        &self,
        source_name: &SourceFullName,
        options: NewSource<'_>,
    ) -> Result<Source> {
        Ok(self
            .put::<_, _, CreateSourceResponse>(
                self.endpoints.source_by_name(source_name)?,
                CreateSourceRequest { source: options },
            )
            .await?
            .source)
    }

    /// Update a source.
    pub async fn update_source(
        &self,
        source_name: &SourceFullName,
        options: UpdateSource<'_>,
    ) -> Result<Source> {
        Ok(self
            .post::<_, _, UpdateSourceResponse>(
                self.endpoints.source_by_name(source_name)?,
                UpdateSourceRequest { source: options },
                Retry::Yes,
            )
            .await?
            .source)
    }

    /// Delete a source.
    pub async fn delete_source(&self, source: impl Into<SourceIdentifier>) -> Result<()> {
        let source_id = match source.into() {
            SourceIdentifier::Id(source_id) => source_id,
            source @ SourceIdentifier::FullName(_) => self.get_source(source).await?.id,
        };
        self.delete(self.endpoints.source_by_id(&source_id)?).await
    }

    /// Delete a user.
    pub async fn delete_user(&self, user: impl Into<UserIdentifier>) -> Result<()> {
        let UserIdentifier::Id(user_id) = user.into();
        self.delete(self.endpoints.user_by_id(&user_id)?).await
    }

    /// Delete comments by id in a source.
    pub async fn delete_comments(
        &self,
        source: impl Into<SourceIdentifier>,
        comments: &[CommentId],
    ) -> Result<()> {
        let source_full_name = match source.into() {
            source @ SourceIdentifier::Id(_) => self.get_source(source).await?.full_name(),
            SourceIdentifier::FullName(source_full_name) => source_full_name,
        };
        self.delete_query(
            self.endpoints.comments_v1(&source_full_name)?,
            Some(&id_list_query(comments.iter().map(|uid| &uid.0))),
        )
        .await
    }

    /// Get a page of comments from a source.
    pub async fn get_comments_iter_page(
        &self,
        source_name: &SourceFullName,
        continuation: Option<&ContinuationKind>,
        to_timestamp: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<CommentsIterPage> {
        // Comments are returned from the API in increasing order of their
        // `timestamp` field.
        let (from_timestamp, after) = match continuation {
            // If we have a timestamp, then this is a request for the first page of
            // a series of comments with timestamps starting from the given time.
            Some(ContinuationKind::Timestamp(from_timestamp)) => (Some(*from_timestamp), None),
            // If we have a continuation, then this is a request for page n+1 of
            // a series of comments, where the continuation came from page n.
            Some(ContinuationKind::Continuation(after)) => (None, Some(after)),
            // Otherwise, this is a request for the first page of a series of comments
            // with timestamps starting from the beginning of time.
            None => (None, None),
        };
        let query_params = GetCommentsIterPageQuery {
            from_timestamp,
            to_timestamp,
            after,
            limit,
        };
        self.get_query(self.endpoints.comments(source_name)?, Some(&query_params))
            .await
    }

    /// Iterate through all comments in a source.
    pub fn get_comments<'a>(
        &'a self,
        source_name: &'a SourceFullName,
        page_size: Option<usize>,
        timerange: CommentsIterTimerange,
    ) -> impl Stream<Item = Result<Vec<Comment>>> + 'a {
        // Default number of comments per page to request from API.
        const DEFAULT_PAGE_SIZE: usize = 64;

        struct CommentCursor {
            continuation: Option<ContinuationKind>,
            page_size: usize,
            to_timestamp: Option<DateTime<Utc>>,
            done: bool,
        }
        let cursor = CommentCursor {
            continuation: timerange.from.map(ContinuationKind::Timestamp),
            page_size: page_size.unwrap_or(DEFAULT_PAGE_SIZE),
            to_timestamp: timerange.to,
            done: false,
        };
        futures::stream::unfold(cursor, |mut cursor| async {
            if cursor.done {
                return None;
            }

            let comments_page = self
                .get_comments_iter_page(
                    source_name,
                    cursor.continuation.as_ref(),
                    cursor.to_timestamp,
                    cursor.page_size,
                )
                .await
                .map(|page| {
                    cursor.continuation = page.continuation.map(ContinuationKind::Continuation);
                    cursor.done = cursor.continuation.is_none();
                    page.comments
                });

            Some((comments_page, cursor))
        })
    }

    /// Get a single comment by id.
    pub async fn get_comment<'a>(
        &'a self,
        source_name: &'a SourceFullName,
        comment_id: &'a CommentId,
    ) -> Result<Comment> {
        Ok(self
            .get::<_, GetCommentResponse>(self.endpoints.comment_by_id(source_name, comment_id)?)
            .await?
            .comment)
    }

    pub async fn put_comments(
        &self,
        source_name: &SourceFullName,
        comments: &[NewComment],
    ) -> Result<PutCommentsResponse> {
        self.put(
            self.endpoints.comments(source_name)?,
            PutCommentsRequest { comments },
        )
        .await
    }

    pub async fn sync_comments(
        &self,
        source_name: &SourceFullName,
        comments: &[NewComment],
    ) -> Result<SyncCommentsResponse> {
        self.post(
            self.endpoints.sync_comments(source_name)?,
            SyncCommentsRequest { comments },
            Retry::Yes,
        )
        .await
    }

    pub async fn put_emails(
        &self,
        bucket_name: &BucketFullName,
        emails: &[NewEmail],
    ) -> Result<PutEmailsResponse> {
        self.put(
            self.endpoints.put_emails(bucket_name)?,
            PutEmailsRequest { emails },
        )
        .await
    }

    pub async fn post_user(&self, user_id: &UserId, user: UpdateUser) -> Result<PostUserResponse> {
        self.post(
            self.endpoints.post_user(user_id)?,
            PostUserRequest { user: &user },
            Retry::Yes,
        )
        .await
    }

    #[cfg(feature = "native")]
    pub async fn put_comment_audio(
        &self,
        source_id: &SourceId,
        comment_id: &CommentId,
        audio_path: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        use reqwest::multipart::{Form, Part};

        let audio_part = {
            let file_name: String = audio_path
                .as_ref()
                .file_name()
                .map(std::ffi::OsStr::to_string_lossy)
                .unwrap_or_else(|| "unknown".into())
                .to_string(); // it's a Cow

            let audio_content =
                tokio::fs::read(audio_path)
                    .await
                    .map_err(|source| Error::Unknown {
                        message: "Could not read audio file {audio_path}".to_owned(),
                        source: source.into(),
                    })?;

            Part::bytes(audio_content).file_name(file_name)
        };

        let http_response = self
            .http_client
            .put(self.endpoints.comment_audio(source_id, comment_id)?)
            .headers(self.headers.clone())
            .multipart(Form::new().part("file", audio_part))
            .send()
            .await
            .map_err(|source| Error::ReqwestError {
                message: "PUT comment audio operation failed".to_owned(),
                source,
            })?;
        let status = http_response.status();
        http_response
            .json::<Response<EmptySuccess>>()
            .await
            .map_err(Error::BadJsonResponse)?
            .into_result(status)?;
        Ok(())
    }

    pub async fn get_datasets(&self) -> Result<Vec<Dataset>> {
        Ok(self
            .get::<_, GetAvailableDatasetsResponse>(self.endpoints.datasets.clone())
            .await?
            .datasets)
    }

    pub async fn get_dataset<IdentifierT>(&self, dataset: IdentifierT) -> Result<Dataset>
    where
        IdentifierT: Into<DatasetIdentifier>,
    {
        Ok(match dataset.into() {
            DatasetIdentifier::Id(dataset_id) => {
                self.get::<_, GetDatasetResponse>(self.endpoints.dataset_by_id(&dataset_id)?)
                    .await?
                    .dataset
            }
            DatasetIdentifier::FullName(dataset_name) => {
                self.get::<_, GetDatasetResponse>(self.endpoints.dataset_by_name(&dataset_name)?)
                    .await?
                    .dataset
            }
        })
    }

    /// Create a dataset.
    pub async fn create_dataset(
        &self,
        dataset_name: &DatasetFullName,
        options: NewDataset<'_>,
    ) -> Result<Dataset> {
        Ok(self
            .put::<_, _, CreateDatasetResponse>(
                self.endpoints.dataset_by_name(dataset_name)?,
                CreateDatasetRequest { dataset: options },
            )
            .await?
            .dataset)
    }

    /// Update a dataset.
    pub async fn update_dataset(
        &self,
        dataset_name: &DatasetFullName,
        options: UpdateDataset<'_>,
    ) -> Result<Dataset> {
        eprintln!(
            "{:?}",
            UpdateDatasetRequest {
                dataset: options.clone()
            }
        );

        Ok(self
            .post::<_, _, UpdateDatasetResponse>(
                self.endpoints.dataset_by_name(dataset_name)?,
                UpdateDatasetRequest { dataset: options },
                Retry::Yes,
            )
            .await?
            .dataset)
    }

    pub async fn delete_dataset<IdentifierT>(&self, dataset: IdentifierT) -> Result<()>
    where
        IdentifierT: Into<DatasetIdentifier>,
    {
        let dataset_id = match dataset.into() {
            DatasetIdentifier::Id(dataset_id) => dataset_id,
            dataset @ DatasetIdentifier::FullName(_) => self.get_dataset(dataset).await?.id,
        };
        self.delete(self.endpoints.dataset_by_id(&dataset_id)?)
            .await
    }

    /// Get labellings for a given a dataset and a list of comment UIDs.
    pub async fn get_labellings<'a>(
        &self,
        dataset_name: &DatasetFullName,
        comment_uids: impl Iterator<Item = &'a CommentUid>,
    ) -> Result<Vec<AnnotatedComment>> {
        Ok(self
            .get_query::<_, _, GetAnnotationsResponse>(
                self.endpoints.get_labellings(dataset_name)?,
                Some(&id_list_query(comment_uids.into_iter().map(|id| &id.0))),
            )
            .await?
            .results)
    }

    /// Iterate through all reviewed comments in a source.
    pub fn get_labellings_iter<'a>(
        &'a self,
        dataset_name: &'a DatasetFullName,
        source_id: &'a SourceId,
        return_predictions: bool,
        limit: Option<usize>,
    ) -> impl Stream<Item = Result<Vec<AnnotatedComment>>> + 'a {
        struct LabellingsCursor {
            after: Option<GetLabellingsAfter>,
            return_predictions: bool,
            limit: Option<usize>,
            done: bool,
        }
        let cursor = LabellingsCursor {
            after: None,
            return_predictions,
            limit,
            done: false,
        };

        futures::stream::unfold(cursor, |mut cursor| async {
            if cursor.done {
                return None;
            }
            let page_result = self
                .get_labellings_in_bulk(
                    dataset_name,
                    GetLabellingsInBulk {
                        source_id,
                        return_predictions: &cursor.return_predictions,
                        after: &cursor.after,
                        limit: &cursor.limit,
                    },
                )
                .await
                .map(|page| {
                    if cursor.after == page.after && !page.results.is_empty() {
                        panic!("Labellings API did not increment pagination continuation");
                    }

                    cursor.after = page.after;
                    if page.results.is_empty() {
                        cursor.done = true;
                    }

                    page.results
                });
            Some((page_result, cursor))
        })
    }

    /// Get reviewed comments in bulk
    pub async fn get_labellings_in_bulk(
        &self,
        dataset_name: &DatasetFullName,
        query_parameters: GetLabellingsInBulk<'_>,
    ) -> Result<GetAnnotationsResponse> {
        self.get_query::<_, _, GetAnnotationsResponse>(
            self.endpoints.get_labellings(dataset_name)?,
            Some(&query_parameters),
        )
        .await
    }

    /// Update labellings for a given a dataset and comment UID.
    pub async fn update_labelling(
        &self,
        dataset_name: &DatasetFullName,
        comment_uid: &CommentUid,
        labelling: Option<&[NewLabelling]>,
        entities: Option<&NewEntities>,
        moon_forms: Option<&[NewMoonForm]>,
    ) -> Result<AnnotatedComment> {
        self.post::<_, _, AnnotatedComment>(
            self.endpoints.post_labelling(dataset_name, comment_uid)?,
            UpdateAnnotationsRequest {
                labelling,
                entities,
                moon_forms,
            },
            Retry::Yes,
        )
        .await
    }

    /// Get predictions for a given a dataset, a model version, and a list of comment UIDs.
    pub async fn get_comment_predictions<'a>(
        &self,
        dataset_name: &DatasetFullName,
        model_version: &ModelVersion,
        comment_uids: impl Iterator<Item = &'a CommentUid>,
    ) -> Result<Vec<Prediction>> {
        Ok(self
            .post::<_, _, GetPredictionsResponse>(
                self.endpoints
                    .get_comment_predictions(dataset_name, model_version)?,
                json!({
                    "threshold": "auto",
                    "uids": comment_uids.into_iter().map(|id| id.0.as_str()).collect::<Vec<_>>(),
                }),
                Retry::Yes,
            )
            .await?
            .predictions)
    }

    pub async fn get_triggers(&self, dataset_name: &DatasetFullName) -> Result<Vec<Trigger>> {
        Ok(self
            .get::<_, GetTriggersResponse>(self.endpoints.triggers(dataset_name)?)
            .await?
            .triggers)
    }

    pub async fn get_recent_comments(
        &self,
        dataset_name: &DatasetFullName,
        filter: &CommentFilter,
        limit: usize,
        continuation: Option<&Continuation>,
    ) -> Result<RecentCommentsPage> {
        self.post::<_, _, RecentCommentsPage>(
            self.endpoints.recent_comments(dataset_name)?,
            GetRecentRequest {
                limit,
                filter,
                continuation,
            },
            Retry::No,
        )
        .await
    }

    pub async fn get_current_user(&self) -> Result<User> {
        Ok(self
            .get::<_, GetCurrentUserResponse>(self.endpoints.current_user.clone())
            .await?
            .user)
    }

    pub async fn get_users(&self) -> Result<Vec<User>> {
        Ok(self
            .get::<_, GetAvailableUsersResponse>(self.endpoints.users.clone())
            .await?
            .users)
    }

    pub async fn create_user(&self, user: NewUser<'_>) -> Result<User> {
        Ok(self
            .put::<_, _, CreateUserResponse>(
                self.endpoints.users.clone(),
                CreateUserRequest { user },
            )
            .await?
            .user)
    }

    pub async fn send_welcome_email(&self, user_id: UserId) -> Result<()> {
        self.post::<_, _, WelcomeEmailResponse>(
            self.endpoints.welcome_email(&user_id)?,
            json!({}),
            Retry::No,
        )
        .await?;
        Ok(())
    }

    pub async fn get_statistics(&self, dataset_name: &DatasetFullName) -> Result<Statistics> {
        Ok(self
            .post::<_, _, GetStatisticsResponse>(
                self.endpoints.statistics(dataset_name)?,
                json!({}),
                Retry::No,
            )
            .await?
            .statistics)
    }

    /// Create a new bucket.
    pub async fn create_bucket(
        &self,
        bucket_name: &BucketFullName,
        options: NewBucket<'_>,
    ) -> Result<Bucket> {
        Ok(self
            .put::<_, _, CreateBucketResponse>(
                self.endpoints.bucket_by_name(bucket_name)?,
                CreateBucketRequest { bucket: options },
            )
            .await?
            .bucket)
    }

    pub async fn get_buckets(&self) -> Result<Vec<Bucket>> {
        Ok(self
            .get::<_, GetAvailableBucketsResponse>(self.endpoints.buckets.clone())
            .await?
            .buckets)
    }

    pub async fn get_bucket<IdentifierT>(&self, bucket: IdentifierT) -> Result<Bucket>
    where
        IdentifierT: Into<BucketIdentifier>,
    {
        Ok(match bucket.into() {
            BucketIdentifier::Id(bucket_id) => {
                self.get::<_, GetBucketResponse>(self.endpoints.bucket_by_id(&bucket_id)?)
                    .await?
                    .bucket
            }
            BucketIdentifier::FullName(bucket_name) => {
                self.get::<_, GetBucketResponse>(self.endpoints.bucket_by_name(&bucket_name)?)
                    .await?
                    .bucket
            }
        })
    }

    pub async fn delete_bucket<IdentifierT>(&self, bucket: IdentifierT) -> Result<()>
    where
        IdentifierT: Into<BucketIdentifier>,
    {
        let bucket_id = match bucket.into() {
            BucketIdentifier::Id(bucket_id) => bucket_id,
            bucket @ BucketIdentifier::FullName(_) => self.get_bucket(bucket).await?.id,
        };
        self.delete(self.endpoints.bucket_by_id(&bucket_id)?).await
    }

    pub async fn fetch_trigger_comments(
        &self,
        trigger_name: &TriggerFullName,
        size: u32,
    ) -> Result<TriggerBatch> {
        self.post(
            self.endpoints.trigger_fetch(trigger_name)?,
            TriggerFetchRequest { size },
            Retry::No,
        )
        .await
    }

    pub async fn advance_trigger(
        &self,
        trigger_name: &TriggerFullName,
        sequence_id: TriggerSequenceId,
    ) -> Result<()> {
        self.post::<_, _, serde::de::IgnoredAny>(
            self.endpoints.trigger_advance(trigger_name)?,
            TriggerAdvanceRequest { sequence_id },
            Retry::No,
        )
        .await?;
        Ok(())
    }

    pub async fn reset_trigger(
        &self,
        trigger_name: &TriggerFullName,
        to_comment_created_at: DateTime<Utc>,
    ) -> Result<()> {
        self.post::<_, _, serde::de::IgnoredAny>(
            self.endpoints.trigger_reset(trigger_name)?,
            TriggerResetRequest {
                to_comment_created_at,
            },
            Retry::No,
        )
        .await?;
        Ok(())
    }

    pub async fn tag_trigger_exceptions<'a>(
        &self,
        trigger_name: &TriggerFullName,
        exceptions: &[TriggerException<'a>],
    ) -> Result<()> {
        self.put::<_, _, serde::de::IgnoredAny>(
            self.endpoints.trigger_exceptions(trigger_name)?,
            TagTriggerExceptionsRequest { exceptions },
        )
        .await?;
        Ok(())
    }

    /// Gets a project.
    pub async fn get_project(&self, project_name: &ProjectName) -> Result<Project> {
        let response = self
            .get::<_, GetProjectResponse>(self.endpoints.project_by_name(project_name)?)
            .await?;
        Ok(response.project)
    }

    /// Gets all projects.
    pub async fn get_projects(&self) -> Result<Vec<Project>> {
        let response = self
            .get::<_, GetProjectsResponse>(self.endpoints.projects.clone())
            .await?;
        Ok(response.projects)
    }

    /// Creates a new project.
    pub async fn create_project<'a>(
        &self,
        project_name: &ProjectName,
        options: NewProject<'a>,
        user_ids: &[UserId],
    ) -> Result<Project> {
        Ok(self
            .put::<_, _, CreateProjectResponse>(
                self.endpoints.project_by_name(project_name)?,
                CreateProjectRequest {
                    project: options,
                    user_ids,
                },
            )
            .await?
            .project)
    }

    /// Updates an existing project.
    pub async fn update_project<'request>(
        &self,
        project_name: &ProjectName,
        options: UpdateProject<'request>,
    ) -> Result<Project> {
        Ok(self
            .post::<_, _, UpdateProjectResponse>(
                self.endpoints.project_by_name(project_name)?,
                UpdateProjectRequest { project: options },
                Retry::Yes,
            )
            .await?
            .project)
    }

    /// Deletes an existing project.
    pub async fn delete_project(
        &self,
        project_name: &ProjectName,
        force_delete: ForceDeleteProject,
    ) -> Result<()> {
        let endpoint = self.endpoints.project_by_name(project_name)?;
        match force_delete {
            ForceDeleteProject::No => self.delete(endpoint).await?,
            ForceDeleteProject::Yes => {
                self.delete_query(endpoint, Some(&json!({ "force": true })))
                    .await?
            }
        };
        Ok(())
    }

    async fn get<LocationT, SuccessT>(&self, url: LocationT) -> Result<SuccessT>
    where
        LocationT: IntoUrl + Display + Clone,
        for<'de> SuccessT: Deserialize<'de>,
    {
        self.get_query::<LocationT, (), SuccessT>(url, None).await
    }

    async fn get_query<LocationT, QueryT, SuccessT>(
        &self,
        url: LocationT,
        query: Option<&QueryT>,
    ) -> Result<SuccessT>
    where
        LocationT: IntoUrl + Display + Clone,
        QueryT: Serialize,
        for<'de> SuccessT: Deserialize<'de>,
    {
        log::debug!("Attempting GET `{}`", url);
        let http_response = self
            .with_retries(|| {
                let mut request = self
                    .http_client
                    .get(url.clone())
                    .headers(self.headers.clone());
                if let Some(query) = query {
                    request = request.query(query);
                }
                request.send()
            })
            .await
            .map_err(|source| Error::ReqwestError {
                source,
                message: "GET operation failed.".to_owned(),
            })?;
        let status = http_response.status();
        http_response
            .json::<Response<SuccessT>>()
            .await
            .map_err(Error::BadJsonResponse)?
            .into_result(status)
    }

    async fn delete<LocationT>(&self, url: LocationT) -> Result<()>
    where
        LocationT: IntoUrl + Display + Clone,
    {
        self.delete_query::<LocationT, ()>(url, None).await
    }

    async fn delete_query<LocationT, QueryT>(
        &self,
        url: LocationT,
        query: Option<&QueryT>,
    ) -> Result<()>
    where
        LocationT: IntoUrl + Display + Clone,
        QueryT: Serialize,
    {
        log::debug!("Attempting DELETE `{}`", url);

        let attempts = Cell::new(0);
        let http_response = self
            .with_retries(|| {
                attempts.set(attempts.get() + 1);

                let mut request = self
                    .http_client
                    .delete(url.clone())
                    .headers(self.headers.clone());
                if let Some(query) = query {
                    request = request.query(query);
                }
                request.send()
            })
            .await
            .map_err(|source| Error::ReqwestError {
                source,
                message: "DELETE operation failed.".to_owned(),
            })?;
        let status = http_response.status();
        http_response
            .json::<Response<EmptySuccess>>()
            .await
            .map_err(Error::BadJsonResponse)?
            .into_result(status)
            .map_or_else(
                // Ignore 404 not found if the request had to be re-tried - assume the target
                // object was deleted on a previous incomplete request.
                |error| {
                    if attempts.get() > 1 && status == reqwest::StatusCode::NOT_FOUND {
                        Ok(())
                    } else {
                        Err(error)
                    }
                },
                |_| Ok(()),
            )
    }

    async fn post<LocationT, RequestT, SuccessT>(
        &self,
        url: LocationT,
        request: RequestT,
        retry: Retry,
    ) -> Result<SuccessT>
    where
        LocationT: IntoUrl + Display + Clone,
        RequestT: Serialize,
        for<'de> SuccessT: Deserialize<'de>,
    {
        log::debug!("Attempting POST `{}`", url);
        let do_request = || {
            self.http_client
                .post(url.clone())
                .headers(self.headers.clone())
                .json(&request)
                .send()
        };
        let result = match retry {
            Retry::Yes => self.with_retries(do_request).await,
            Retry::No => do_request().await,
        };

        let http_response = result.map_err(|source| Error::ReqwestError {
            source,
            message: "POST operation failed.".to_owned(),
        })?;
        let status = http_response.status();
        http_response
            .json::<Response<SuccessT>>()
            .await
            .map_err(Error::BadJsonResponse)?
            .into_result(status)
    }

    async fn put<LocationT, RequestT, SuccessT>(
        &self,
        url: LocationT,
        request: RequestT,
    ) -> Result<SuccessT>
    where
        LocationT: IntoUrl + Display + Clone,
        RequestT: Serialize,
        for<'de> SuccessT: Deserialize<'de>,
    {
        log::debug!("Attempting PUT `{}`", url);
        let http_response = self
            .with_retries(|| {
                self.http_client
                    .put(url.clone())
                    .headers(self.headers.clone())
                    .json(&request)
                    .send()
            })
            .await
            .map_err(|source| Error::ReqwestError {
                source,
                message: "PUT operation failed.".to_owned(),
            })?;
        let status = http_response.status();
        http_response
            .json::<Response<SuccessT>>()
            .await
            .map_err(Error::BadJsonResponse)?
            .into_result(status)
    }

    async fn with_retries<CallbackT, ResponseT>(
        &self,
        send_request: CallbackT,
    ) -> ReqwestResult<HttpResponse>
    where
        CallbackT: Fn() -> ResponseT,
        ResponseT: Future<Output = ReqwestResult<HttpResponse>>,
    {
        match &self.retrier {
            Some(retrier) => retrier.with_retries(send_request).await,
            None => send_request().await,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Retry {
    Yes,
    No,
}

#[derive(Clone, Debug)]
pub enum ContinuationKind {
    Timestamp(DateTime<Utc>),
    Continuation(Continuation),
}

#[derive(Clone, Debug, Default)]
pub struct CommentsIterTimerange {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug)]
struct Endpoints {
    base: Url,
    datasets: Url,
    sources: Url,
    buckets: Url,
    users: Url,
    current_user: Url,
    projects: Url,
}

fn construct_endpoint(base: &Url, segments: &[&str]) -> Result<Url> {
    let mut endpoint = base.clone();

    let mut endpoint_segments = endpoint
        .path_segments_mut()
        .map_err(|_| Error::BadEndpoint {
            endpoint: base.clone(),
        })?;

    for segment in segments {
        endpoint_segments.push(segment);
    }

    drop(endpoint_segments);
    Ok(endpoint)
}

impl Endpoints {
    pub fn new(base: Url) -> Result<Self> {
        let datasets = construct_endpoint(&base, &["api", "v1", "datasets"])?;
        let sources = construct_endpoint(&base, &["api", "v1", "sources"])?;
        let buckets = construct_endpoint(&base, &["api", "_private", "buckets"])?;
        let users = construct_endpoint(&base, &["api", "_private", "users"])?;
        let current_user = construct_endpoint(&base, &["auth", "user"])?;
        let projects = construct_endpoint(&base, &["api", "_private", "projects"])?;

        Ok(Endpoints {
            base,
            datasets,
            sources,
            buckets,
            users,
            current_user,
            projects,
        })
    }

    fn triggers(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "v1", "datasets", &dataset_name.0, "triggers"],
        )
    }

    fn trigger_fetch(&self, trigger_name: &TriggerFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "datasets",
                &trigger_name.dataset.0,
                &trigger_name.trigger.0,
            ],
        )
    }

    fn trigger_advance(&self, trigger_name: &TriggerFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "datasets",
                &trigger_name.dataset.0,
                "triggers",
                &trigger_name.trigger.0,
            ],
        )
    }

    fn trigger_reset(&self, trigger_name: &TriggerFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "datasets",
                &trigger_name.dataset.0,
                "triggers",
                &trigger_name.trigger.0,
                "reset",
            ],
        )
    }

    fn trigger_exceptions(&self, trigger_name: &TriggerFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "datasets",
                &trigger_name.dataset.0,
                "triggers",
                &trigger_name.trigger.0,
                "exceptions",
            ],
        )
    }

    fn recent_comments(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "datasets", &dataset_name.0, "recent"],
        )
    }

    fn statistics(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "datasets", &dataset_name.0, "statistics"],
        )
    }

    fn user_by_id(&self, user_id: &UserId) -> Result<Url> {
        construct_endpoint(&self.base, &["api", "_private", "users", &user_id.0])
    }

    fn source_by_id(&self, source_id: &SourceId) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "v1", "sources", &format!("id:{}", source_id.0)],
        )
    }

    fn source_by_name(&self, source_name: &SourceFullName) -> Result<Url> {
        construct_endpoint(&self.base, &["api", "v1", "sources", &source_name.0])
    }

    fn comments(&self, source_name: &SourceFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "sources", &source_name.0, "comments"],
        )
    }

    fn comment_by_id(&self, source_name: &SourceFullName, comment_id: &CommentId) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "sources",
                &source_name.0,
                "comments",
                &comment_id.0,
            ],
        )
    }

    fn comments_v1(&self, source_name: &SourceFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "v1", "sources", &source_name.0, "comments"],
        )
    }

    fn sync_comments(&self, source_name: &SourceFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "v1", "sources", &source_name.0, "sync"],
        )
    }

    #[cfg(feature = "native")]
    fn comment_audio(&self, source_id: &SourceId, comment_id: &CommentId) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "_private",
                "sources",
                &format!("id:{}", source_id.0),
                "comments",
                &comment_id.0,
                "audio",
            ],
        )
    }

    fn put_emails(&self, bucket_name: &BucketFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "buckets", &bucket_name.0, "emails"],
        )
    }

    fn post_user(&self, user_id: &UserId) -> Result<Url> {
        construct_endpoint(&self.base, &["api", "_private", "users", &user_id.0])
    }

    fn dataset_by_id(&self, dataset_id: &DatasetId) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "v1", "datasets", &format!("id:{}", dataset_id.0)],
        )
    }

    fn dataset_by_name(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        construct_endpoint(&self.base, &["api", "v1", "datasets", &dataset_name.0])
    }

    fn get_labellings(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "datasets", &dataset_name.0, "labellings"],
        )
    }

    fn get_comment_predictions(
        &self,
        dataset_name: &DatasetFullName,
        model_version: &ModelVersion,
    ) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "datasets",
                &dataset_name.0,
                "labellers",
                &model_version.0.to_string(),
                "predict-comments",
            ],
        )
    }

    fn post_labelling(
        &self,
        dataset_name: &DatasetFullName,
        comment_uid: &CommentUid,
    ) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "_private",
                "datasets",
                &dataset_name.0,
                "labellings",
                &comment_uid.0,
            ],
        )
    }

    fn bucket_by_id(&self, bucket_id: &BucketId) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "buckets", &format!("id:{}", bucket_id.0)],
        )
    }

    fn bucket_by_name(&self, bucket_name: &BucketFullName) -> Result<Url> {
        construct_endpoint(&self.base, &["api", "_private", "buckets", &bucket_name.0])
    }

    fn project_by_name(&self, project_name: &ProjectName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "projects", &project_name.0],
        )
    }

    fn welcome_email(&self, user_id: &UserId) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "users", &user_id.0, "welcome-email"],
        )
    }
}

fn build_headers(config: &Config) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", &config.token.0)).map_err(|_| {
            Error::BadToken {
                token: config.token.0.clone(),
            }
        })?,
    );
    Ok(headers)
}

fn id_list_query<'a>(ids: impl Iterator<Item = &'a String>) -> Vec<(&'static str, &'a str)> {
    // Return a list of pairs ("id", "a"), ("id", "b"), ...
    // The http client will turn this into a query string of
    // the form "id=a&id=b&..."
    ids.map(|id| ("id", id.as_str())).collect()
}

pub static DEFAULT_ENDPOINT: Lazy<Url> =
    Lazy::new(|| Url::parse("https://reinfer.io").expect("Default URL is well-formed"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_list_query() {
        assert_eq!(id_list_query(Vec::new().iter()), Vec::new());
        assert_eq!(
            id_list_query(vec!["foo".to_owned()].iter()),
            vec![("id", "foo")]
        );
        assert_eq!(
            id_list_query(
                vec![
                    "Stream".to_owned(),
                    "River".to_owned(),
                    "Waterfall".to_owned()
                ]
                .iter()
            ),
            vec![("id", "Stream"), ("id", "River"), ("id", "Waterfall"),]
        );
    }
}

/// Maximum number of comments per page which can be requested from the API.
pub const MAX_PAGE_SIZE: usize = 256;
