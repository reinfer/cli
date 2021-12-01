#![deny(clippy::all)]
mod error;
pub mod resources;
pub mod retry;

use chrono::{DateTime, Utc};
use log::debug;
use once_cell::sync::Lazy;
use reqwest::{
    blocking::{multipart::Form, Client as HttpClient, Response as HttpResponse},
    header::{self, HeaderMap, HeaderValue},
    IntoUrl, Proxy, Result as ReqwestResult,
};
use resources::project::ForceDeleteProject;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{cell::Cell, fmt::Display, path::Path};
use url::Url;

use crate::resources::{
    bucket::{
        CreateRequest as CreateBucketRequest, CreateResponse as CreateBucketResponse,
        GetAvailableResponse as GetAvailableBucketsResponse, GetResponse as GetBucketResponse,
    },
    comment::{
        GetAnnotationsResponse, GetCommentResponse, GetLabellingsAfter, GetRecentRequest,
        PutCommentsRequest, PutCommentsResponse, RecentCommentsPage, SyncCommentsRequest,
        UpdateAnnotationsRequest,
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
    },
    user::{
        CreateRequest as CreateUserRequest, CreateResponse as CreateUserResponse,
        GetAvailableResponse as GetAvailableUsersResponse,
        GetCurrentResponse as GetCurrentUserResponse,
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
            EitherLabelling, Entity, HasAnnotations, Id as CommentId, Label, Message, MessageBody,
            MessageSignature, MessageSubject, NewAnnotatedComment, NewComment, NewEntities,
            NewLabelling, PropertyMap, PropertyValue, Sentiment, SyncCommentsResponse,
            Uid as CommentUid,
        },
        dataset::{
            Dataset, FullName as DatasetFullName, Id as DatasetId, Identifier as DatasetIdentifier,
            Name as DatasetName, NewDataset, UpdateDataset,
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
            Name as SourceName, NewSource, Source, SourceType, UpdateSource,
        },
        statistics::Statistics,
        trigger::{
            Batch as TriggerBatch, FullName as TriggerFullName, SequenceId as TriggerSequenceId,
            Trigger,
        },
        user::{
            Email, GlobalPermission, Id as UserId, Identifier as UserIdentifier,
            ModifiedPermissions, NewUser, Organisation, OrganisationPermission, User, Username,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token(pub String);

pub struct Config {
    pub endpoint: Url,
    pub token: Token,
    pub accept_invalid_certificates: bool,
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
        let http_client = build_http_client(&config)?;
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
    pub fn get_sources(&self) -> Result<Vec<Source>> {
        Ok(self
            .get::<_, GetAvailableSourcesResponse>(self.endpoints.sources.clone())?
            .sources)
    }

    /// Get a source by either id or name.
    pub fn get_source(&self, source: impl Into<SourceIdentifier>) -> Result<Source> {
        Ok(match source.into() {
            SourceIdentifier::Id(source_id) => {
                self.get::<_, GetSourceResponse>(self.endpoints.source_by_id(&source_id)?)?
                    .source
            }
            SourceIdentifier::FullName(source_name) => {
                self.get::<_, GetSourceResponse>(self.endpoints.source_by_name(&source_name)?)?
                    .source
            }
        })
    }

    /// Create a new source.
    pub fn create_source(
        &self,
        source_name: &SourceFullName,
        options: NewSource<'_>,
    ) -> Result<Source> {
        Ok(self
            .put::<_, _, CreateSourceResponse>(
                self.endpoints.source_by_name(source_name)?,
                CreateSourceRequest { source: options },
            )?
            .source)
    }

    /// Update a source.
    pub fn update_source(
        &self,
        source_name: &SourceFullName,
        options: UpdateSource<'_>,
    ) -> Result<Source> {
        Ok(self
            .post::<_, _, UpdateSourceResponse>(
                self.endpoints.source_by_name(source_name)?,
                UpdateSourceRequest { source: options },
                Retry::Yes,
            )?
            .source)
    }

    /// Delete a source.
    pub fn delete_source(&self, source: impl Into<SourceIdentifier>) -> Result<()> {
        let source_id = match source.into() {
            SourceIdentifier::Id(source_id) => source_id,
            source @ SourceIdentifier::FullName(_) => self.get_source(source)?.id,
        };
        self.delete::<_>(self.endpoints.source_by_id(&source_id)?)
    }

    /// Delete comments by id in a source.
    pub fn delete_comments(
        &self,
        source: impl Into<SourceIdentifier>,
        comments: &[CommentId],
    ) -> Result<()> {
        let source_full_name = match source.into() {
            source @ SourceIdentifier::Id(_) => self.get_source(source)?.full_name(),
            SourceIdentifier::FullName(source_full_name) => source_full_name,
        };
        self.delete_query::<_, _>(
            self.endpoints.comments_v1(&source_full_name)?,
            Some(&id_list_query(comments.iter().map(|uid| &uid.0))),
        )
    }

    /// Get a page of comments from a source.
    pub fn get_comments_iter_page(
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
        self.get_query::<_, _, _>(self.endpoints.comments(source_name)?, Some(&query_params))
    }

    /// Iterate through all comments in a source.
    pub fn get_comments_iter<'a>(
        &'a self,
        source_name: &'a SourceFullName,
        page_size: Option<usize>,
        timerange: CommentsIterTimerange,
    ) -> CommentsIter<'a> {
        CommentsIter::new(self, source_name, page_size, timerange)
    }

    /// Get a single comment by id.
    pub fn get_comment<'a>(
        &'a self,
        source_name: &'a SourceFullName,
        comment_id: &'a CommentId,
    ) -> Result<Comment> {
        Ok(self
            .get::<_, GetCommentResponse>(self.endpoints.comment_by_id(source_name, comment_id)?)?
            .comment)
    }

    pub fn put_comments(
        &self,
        source_name: &SourceFullName,
        comments: &[NewComment],
    ) -> Result<PutCommentsResponse> {
        self.put::<_, _, _>(
            self.endpoints.comments(source_name)?,
            PutCommentsRequest { comments },
        )
    }

    pub fn sync_comments(
        &self,
        source_name: &SourceFullName,
        comments: &[NewComment],
    ) -> Result<SyncCommentsResponse> {
        self.post::<_, _, _>(
            self.endpoints.sync_comments(source_name)?,
            SyncCommentsRequest { comments },
            Retry::Yes,
        )
    }

    pub fn put_emails(
        &self,
        bucket_name: &BucketFullName,
        emails: &[NewEmail],
    ) -> Result<PutEmailsResponse> {
        self.put::<_, _, _>(
            self.endpoints.put_emails(bucket_name)?,
            PutEmailsRequest { emails },
        )
    }

    pub fn put_comment_audio(
        &self,
        source_id: &SourceId,
        comment_id: &CommentId,
        audio_path: impl AsRef<Path>,
    ) -> Result<()> {
        let form = Form::new()
            .file("file", audio_path)
            .map_err(|source| Error::Unknown {
                message: "PUT comment audio operation failed".to_owned(),
                source: source.into(),
            })?;
        let http_response = self
            .http_client
            .put(self.endpoints.comment_audio(source_id, comment_id)?)
            .headers(self.headers.clone())
            .multipart(form)
            .send()
            .map_err(|source| Error::ReqwestError {
                message: "PUT comment audio operation failed".to_owned(),
                source,
            })?;
        let status = http_response.status();
        http_response
            .json::<Response<EmptySuccess>>()
            .map_err(Error::BadJsonResponse)?
            .into_result(status)?;
        Ok(())
    }

    pub fn get_datasets(&self) -> Result<Vec<Dataset>> {
        Ok(self
            .get::<_, GetAvailableDatasetsResponse>(self.endpoints.datasets.clone())?
            .datasets)
    }

    pub fn get_dataset<IdentifierT>(&self, dataset: IdentifierT) -> Result<Dataset>
    where
        IdentifierT: Into<DatasetIdentifier>,
    {
        Ok(match dataset.into() {
            DatasetIdentifier::Id(dataset_id) => {
                self.get::<_, GetDatasetResponse>(self.endpoints.dataset_by_id(&dataset_id)?)?
                    .dataset
            }
            DatasetIdentifier::FullName(dataset_name) => {
                self.get::<_, GetDatasetResponse>(self.endpoints.dataset_by_name(&dataset_name)?)?
                    .dataset
            }
        })
    }

    /// Create a dataset.
    pub fn create_dataset(
        &self,
        dataset_name: &DatasetFullName,
        options: NewDataset<'_>,
    ) -> Result<Dataset> {
        Ok(self
            .put::<_, _, CreateDatasetResponse>(
                self.endpoints.dataset_by_name(dataset_name)?,
                CreateDatasetRequest { dataset: options },
            )?
            .dataset)
    }

    /// Update a dataset.
    pub fn update_dataset(
        &self,
        dataset_name: &DatasetFullName,
        options: UpdateDataset<'_>,
    ) -> Result<Dataset> {
        Ok(self
            .post::<_, _, UpdateDatasetResponse>(
                self.endpoints.dataset_by_name(dataset_name)?,
                UpdateDatasetRequest { dataset: options },
                Retry::Yes,
            )?
            .dataset)
    }

    pub fn delete_dataset<IdentifierT>(&self, dataset: IdentifierT) -> Result<()>
    where
        IdentifierT: Into<DatasetIdentifier>,
    {
        let dataset_id = match dataset.into() {
            DatasetIdentifier::Id(dataset_id) => dataset_id,
            dataset @ DatasetIdentifier::FullName(_) => self.get_dataset(dataset)?.id,
        };
        self.delete::<_>(self.endpoints.dataset_by_id(&dataset_id)?)
    }

    /// Get labellings for a given a dataset and a list of comment UIDs.
    pub fn get_labellings<'a>(
        &self,
        dataset_name: &DatasetFullName,
        comment_uids: impl Iterator<Item = &'a CommentUid>,
    ) -> Result<Vec<AnnotatedComment>> {
        Ok(self
            .get_query::<_, _, GetAnnotationsResponse>(
                self.endpoints.get_labellings(dataset_name)?,
                Some(&id_list_query(comment_uids.into_iter().map(|id| &id.0))),
            )?
            .results)
    }

    /// Iterate through all reviewed comments in a source.
    pub fn get_labellings_iter<'a>(
        &'a self,
        dataset_name: &'a DatasetFullName,
        source_id: &'a SourceId,
        return_predictions: bool,
        limit: Option<usize>,
    ) -> LabellingsIter<'a> {
        LabellingsIter::new(self, dataset_name, source_id, return_predictions, limit)
    }

    /// Get reviewed comments in bulk
    pub fn get_labellings_in_bulk(
        &self,
        dataset_name: &DatasetFullName,
        query_parameters: GetLabellingsInBulk<'_>,
    ) -> Result<GetAnnotationsResponse> {
        self.get_query::<_, _, GetAnnotationsResponse>(
            self.endpoints.get_labellings(dataset_name)?,
            Some(&query_parameters),
        )
    }

    /// Update labellings for a given a dataset and comment UID.
    pub fn update_labelling(
        &self,
        dataset_name: &DatasetFullName,
        comment_uid: &CommentUid,
        labelling: Option<&[NewLabelling]>,
        entities: Option<&NewEntities>,
    ) -> Result<AnnotatedComment> {
        self.post::<_, _, AnnotatedComment>(
            self.endpoints.post_labelling(dataset_name, comment_uid)?,
            UpdateAnnotationsRequest {
                labelling,
                entities,
            },
            Retry::No,
        )
    }

    pub fn get_triggers(&self, dataset_name: &DatasetFullName) -> Result<Vec<Trigger>> {
        Ok(self
            .get::<_, GetTriggersResponse>(self.endpoints.triggers(dataset_name)?)?
            .triggers)
    }

    pub fn get_recent_comments(
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
    }

    pub fn get_current_user(&self) -> Result<User> {
        Ok(self
            .get::<_, GetCurrentUserResponse>(self.endpoints.current_user.clone())?
            .user)
    }

    pub fn get_users(&self) -> Result<Vec<User>> {
        Ok(self
            .get::<_, GetAvailableUsersResponse>(self.endpoints.users.clone())?
            .users)
    }

    pub fn create_user(&self, user: NewUser<'_>) -> Result<User> {
        Ok(self
            .put::<_, _, CreateUserResponse>(
                self.endpoints.users.clone(),
                CreateUserRequest { user },
            )?
            .user)
    }

    pub fn get_statistics(&self, dataset_name: &DatasetFullName) -> Result<Statistics> {
        Ok(self
            .post::<_, _, GetStatisticsResponse>(
                self.endpoints.statistics(dataset_name)?,
                json!({}),
                Retry::No,
            )?
            .statistics)
    }

    /// Create a new bucket.
    pub fn create_bucket(
        &self,
        bucket_name: &BucketFullName,
        options: NewBucket<'_>,
    ) -> Result<Bucket> {
        Ok(self
            .put::<_, _, CreateBucketResponse>(
                self.endpoints.bucket_by_name(bucket_name)?,
                CreateBucketRequest { bucket: options },
            )?
            .bucket)
    }

    pub fn get_buckets(&self) -> Result<Vec<Bucket>> {
        Ok(self
            .get::<_, GetAvailableBucketsResponse>(self.endpoints.buckets.clone())?
            .buckets)
    }

    pub fn get_bucket<IdentifierT>(&self, bucket: IdentifierT) -> Result<Bucket>
    where
        IdentifierT: Into<BucketIdentifier>,
    {
        Ok(match bucket.into() {
            BucketIdentifier::Id(bucket_id) => {
                self.get::<_, GetBucketResponse>(self.endpoints.bucket_by_id(&bucket_id)?)?
                    .bucket
            }
            BucketIdentifier::FullName(bucket_name) => {
                self.get::<_, GetBucketResponse>(self.endpoints.bucket_by_name(&bucket_name)?)?
                    .bucket
            }
        })
    }

    pub fn delete_bucket<IdentifierT>(&self, bucket: IdentifierT) -> Result<()>
    where
        IdentifierT: Into<BucketIdentifier>,
    {
        let bucket_id = match bucket.into() {
            BucketIdentifier::Id(bucket_id) => bucket_id,
            bucket @ BucketIdentifier::FullName(_) => self.get_bucket(bucket)?.id,
        };
        self.delete::<_>(self.endpoints.bucket_by_id(&bucket_id)?)
    }

    pub fn fetch_trigger_comments(
        &self,
        trigger_name: &TriggerFullName,
        size: u32,
    ) -> Result<TriggerBatch> {
        self.post::<_, _, _>(
            self.endpoints.trigger_fetch(trigger_name)?,
            TriggerFetchRequest { size },
            Retry::No,
        )
    }

    pub fn advance_trigger(
        &self,
        trigger_name: &TriggerFullName,
        sequence_id: TriggerSequenceId,
    ) -> Result<()> {
        self.post::<_, _, serde::de::IgnoredAny>(
            self.endpoints.trigger_advance(trigger_name)?,
            TriggerAdvanceRequest { sequence_id },
            Retry::No,
        )?;
        Ok(())
    }

    pub fn reset_trigger(
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
        )?;
        Ok(())
    }

    /// Gets a project.
    pub fn get_project(&self, project_name: &ProjectName) -> Result<Project> {
        let response =
            self.get::<_, GetProjectResponse>(self.endpoints.project_by_name(project_name)?)?;
        Ok(response.project)
    }

    /// Gets all projects.
    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let response = self.get::<_, GetProjectsResponse>(self.endpoints.projects.clone())?;
        Ok(response.projects)
    }

    /// Creates a new project.
    pub fn create_project(
        &self,
        project_name: &ProjectName,
        options: NewProject,
        user_ids: &[UserId],
    ) -> Result<Project> {
        Ok(self
            .put::<_, _, CreateProjectResponse>(
                self.endpoints.project_by_name(project_name)?,
                CreateProjectRequest {
                    project: options,
                    user_ids,
                },
            )?
            .project)
    }

    /// Updates an existing project.
    pub fn update_project(
        &self,
        project_name: &ProjectName,
        options: UpdateProject,
    ) -> Result<Project> {
        Ok(self
            .post::<_, _, UpdateProjectResponse>(
                self.endpoints.project_by_name(project_name)?,
                UpdateProjectRequest { project: options },
                Retry::Yes,
            )?
            .project)
    }

    /// Deletes an existing project.
    pub fn delete_project(
        &self,
        project_name: &ProjectName,
        force_delete: ForceDeleteProject,
    ) -> Result<()> {
        let endpoint = self.endpoints.project_by_name(project_name)?;
        match force_delete {
            ForceDeleteProject::No => self.delete(endpoint)?,
            ForceDeleteProject::Yes => {
                self.delete_query(endpoint, Some(&json!({ "force": true })))?
            }
        };
        Ok(())
    }

    fn get<LocationT, SuccessT>(&self, url: LocationT) -> Result<SuccessT>
    where
        LocationT: IntoUrl + Display + Clone,
        for<'de> SuccessT: Deserialize<'de>,
    {
        self.get_query::<LocationT, (), SuccessT>(url, None)
    }

    fn get_query<LocationT, QueryT, SuccessT>(
        &self,
        url: LocationT,
        query: Option<&QueryT>,
    ) -> Result<SuccessT>
    where
        LocationT: IntoUrl + Display + Clone,
        QueryT: Serialize,
        for<'de> SuccessT: Deserialize<'de>,
    {
        debug!("Attempting GET `{}`", url);
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
            .map_err(|source| Error::ReqwestError {
                source,
                message: "GET operation failed.".to_owned(),
            })?;
        let status = http_response.status();
        http_response
            .json::<Response<SuccessT>>()
            .map_err(Error::BadJsonResponse)?
            .into_result(status)
    }

    fn delete<LocationT>(&self, url: LocationT) -> Result<()>
    where
        LocationT: IntoUrl + Display + Clone,
    {
        self.delete_query::<LocationT, ()>(url, None)
    }

    fn delete_query<LocationT, QueryT>(&self, url: LocationT, query: Option<&QueryT>) -> Result<()>
    where
        LocationT: IntoUrl + Display + Clone,
        QueryT: Serialize,
    {
        debug!("Attempting DELETE `{}`", url);

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
            .map_err(|source| Error::ReqwestError {
                source,
                message: "DELETE operation failed.".to_owned(),
            })?;
        let status = http_response.status();
        http_response
            .json::<Response<EmptySuccess>>()
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

    fn post<LocationT, RequestT, SuccessT>(
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
        debug!("Attempting POST `{}`", url);
        let do_request = || {
            self.http_client
                .post(url.clone())
                .headers(self.headers.clone())
                .json(&request)
                .send()
        };
        let result = match retry {
            Retry::Yes => self.with_retries(do_request),
            Retry::No => do_request(),
        };

        let http_response = result.map_err(|source| Error::ReqwestError {
            source,
            message: "POST operation failed.".to_owned(),
        })?;
        let status = http_response.status();
        http_response
            .json::<Response<SuccessT>>()
            .map_err(Error::BadJsonResponse)?
            .into_result(status)
    }

    fn put<LocationT, RequestT, SuccessT>(
        &self,
        url: LocationT,
        request: RequestT,
    ) -> Result<SuccessT>
    where
        LocationT: IntoUrl + Display + Clone,
        RequestT: Serialize,
        for<'de> SuccessT: Deserialize<'de>,
    {
        debug!("Attempting PUT `{}`", url);
        let http_response = self
            .with_retries(|| {
                self.http_client
                    .put(url.clone())
                    .headers(self.headers.clone())
                    .json(&request)
                    .send()
            })
            .map_err(|source| Error::ReqwestError {
                source,
                message: "PUT operation failed.".to_owned(),
            })?;
        let status = http_response.status();
        http_response
            .json::<Response<SuccessT>>()
            .map_err(Error::BadJsonResponse)?
            .into_result(status)
    }

    fn with_retries(
        &self,
        send_request: impl Fn() -> ReqwestResult<HttpResponse>,
    ) -> ReqwestResult<HttpResponse> {
        match &self.retrier {
            Some(retrier) => retrier.with_retries(send_request),
            None => send_request(),
        }
    }
}

enum Retry {
    Yes,
    No,
}

pub enum ContinuationKind {
    Timestamp(DateTime<Utc>),
    Continuation(Continuation),
}

pub struct CommentsIter<'a> {
    client: &'a Client,
    source_name: &'a SourceFullName,
    continuation: Option<ContinuationKind>,
    done: bool,
    page_size: usize,
    to_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
pub struct CommentsIterTimerange {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}
impl<'a> CommentsIter<'a> {
    // Default number of comments per page to request from API.
    pub const DEFAULT_PAGE_SIZE: usize = 64;
    // Maximum number of comments per page which can be requested from the API.
    pub const MAX_PAGE_SIZE: usize = 256;

    fn new(
        client: &'a Client,
        source_name: &'a SourceFullName,
        page_size: Option<usize>,
        timerange: CommentsIterTimerange,
    ) -> Self {
        let (from_timestamp, to_timestamp) = (timerange.from, timerange.to);
        Self {
            client,
            source_name,
            to_timestamp,
            continuation: from_timestamp.map(ContinuationKind::Timestamp),
            done: false,
            page_size: page_size.unwrap_or(Self::DEFAULT_PAGE_SIZE),
        }
    }
}

impl<'a> Iterator for CommentsIter<'a> {
    type Item = Result<Vec<Comment>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let response = self.client.get_comments_iter_page(
            self.source_name,
            self.continuation.as_ref(),
            self.to_timestamp,
            self.page_size,
        );
        Some(response.map(|page| {
            self.continuation = page.continuation.map(ContinuationKind::Continuation);
            self.done = self.continuation.is_none();
            page.comments
        }))
    }
}

pub struct LabellingsIter<'a> {
    client: &'a Client,
    dataset_name: &'a DatasetFullName,
    source_id: &'a SourceId,
    return_predictions: bool,
    after: Option<GetLabellingsAfter>,
    limit: Option<usize>,
    done: bool,
}

impl<'a> LabellingsIter<'a> {
    fn new(
        client: &'a Client,
        dataset_name: &'a DatasetFullName,
        source_id: &'a SourceId,
        return_predictions: bool,
        limit: Option<usize>,
    ) -> Self {
        Self {
            client,
            dataset_name,
            source_id,
            return_predictions,
            after: None,
            limit,
            done: false,
        }
    }
}

impl<'a> Iterator for LabellingsIter<'a> {
    type Item = Result<Vec<AnnotatedComment>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let response = self.client.get_labellings_in_bulk(
            self.dataset_name,
            GetLabellingsInBulk {
                source_id: self.source_id,
                return_predictions: &self.return_predictions,
                after: &self.after,
                limit: &self.limit,
            },
        );
        Some(response.map(|page| {
            if self.after == page.after && !page.results.is_empty() {
                panic!("Labellings API did not increment pagination continuation");
            }
            self.after = page.after;
            if page.results.is_empty() {
                self.done = true;
            }
            page.results
        }))
    }
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

impl Endpoints {
    pub fn new(base: Url) -> Result<Self> {
        let datasets = base
            .join("/api/v1/datasets")
            .map_err(|source| Error::UrlParseError {
                source,
                message: "Could not build URL for dataset resources.".to_owned(),
            })?;
        let sources = base
            .join("/api/v1/sources")
            .map_err(|source| Error::UrlParseError {
                source,
                message: "Could not build URL for source resources.".to_owned(),
            })?;
        let buckets =
            base.join("/api/_private/buckets")
                .map_err(|source| Error::UrlParseError {
                    source,
                    message: "Could not build URL for bucket resources.".to_owned(),
                })?;
        let users = base
            .join("/api/_private/users")
            .map_err(|source| Error::UrlParseError {
                source,
                message: "Could not build URL for users resources.".to_owned(),
            })?;
        let current_user = base
            .join("/auth/user")
            .map_err(|source| Error::UrlParseError {
                source,
                message: "Could not build URL for users resources.".to_owned(),
            })?;
        let projects =
            base.join("/api/_private/projects")
                .map_err(|source| Error::UrlParseError {
                    source,
                    message: "Could not build URL for project resources.".to_owned(),
                })?;
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
        self.base
            .join(&format!("/api/v1/datasets/{}/triggers", dataset_name.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: "Could not build URL to get trigger resources.".to_owned(),
            })
    }

    fn trigger_fetch(&self, trigger_name: &TriggerFullName) -> Result<Url> {
        self.base
            .join(&format!(
                "/api/v1/datasets/{}/triggers/{}/fetch",
                trigger_name.dataset.0, trigger_name.trigger.0
            ))
            .map_err(|source| Error::UrlParseError {
                source,
                message: "Could not build URL to fetch trigger results.".to_owned(),
            })
    }

    fn trigger_advance(&self, trigger_name: &TriggerFullName) -> Result<Url> {
        self.base
            .join(&format!(
                "/api/v1/datasets/{}/triggers/{}/advance",
                trigger_name.dataset.0, trigger_name.trigger.0
            ))
            .map_err(|source| Error::UrlParseError {
                source,
                message: "Could not build URL to advance triggers.".to_owned(),
            })
    }

    fn trigger_reset(&self, trigger_name: &TriggerFullName) -> Result<Url> {
        self.base
            .join(&format!(
                "/api/v1/datasets/{}/triggers/{}/reset",
                trigger_name.dataset.0, trigger_name.trigger.0
            ))
            .map_err(|source| Error::UrlParseError {
                source,
                message: "Could not build URL to reset triggers.".to_owned(),
            })
    }

    fn recent_comments(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        self.base
            .join(&format!("/api/_private/datasets/{}/recent", dataset_name.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: "Could not build URL for recent comments query.".to_owned(),
            })
    }

    fn statistics(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        self.base
            .join(&format!(
                "/api/_private/datasets/{}/statistics",
                dataset_name.0
            ))
            .map_err(|source| Error::UrlParseError {
                source,
                message: "Could not build URL for dataset statistics query.".to_owned(),
            })
    }

    fn source_by_id(&self, source_id: &SourceId) -> Result<Url> {
        self.base
            .join(&format!("/api/v1/sources/id:{}", source_id.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build URL for source resource with id `{}`.",
                    source_id.0
                ),
            })
    }

    fn source_by_name(&self, source_name: &SourceFullName) -> Result<Url> {
        self.base
            .join(&format!("/api/v1/sources/{}", source_name.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build URL for source resource with name `{}`.",
                    source_name.0
                ),
            })
    }

    fn comments(&self, source_name: &SourceFullName) -> Result<Url> {
        self.base
            .join(&format!("/api/_private/sources/{}/comments", source_name.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build get comments URL for source `{}`.",
                    source_name.0,
                ),
            })
    }

    fn comment_by_id(&self, source_name: &SourceFullName, comment_id: &CommentId) -> Result<Url> {
        self.base
            .join(&format!(
                "/api/v1/sources/{}/comments/{}",
                source_name.0, comment_id.0
            ))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build get comment by id URL for source `{}`, comment `{}`.",
                    source_name.0, comment_id.0,
                ),
            })
    }

    fn comments_v1(&self, source_name: &SourceFullName) -> Result<Url> {
        self.base
            .join(&format!("/api/v1/sources/{}/comments", source_name.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build get comments v1 URL for source `{}`.",
                    source_name.0,
                ),
            })
    }

    fn sync_comments(&self, source_name: &SourceFullName) -> Result<Url> {
        self.base
            .join(&format!("/api/v1/sources/{}/sync", source_name.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!("Could not build sync URL for source `{}`.", source_name.0),
            })
    }

    fn comment_audio(&self, source_id: &SourceId, comment_id: &CommentId) -> Result<Url> {
        self.base
            .join(&format!(
                "/api/_private/sources/id:{}/comments/{}/audio",
                source_id.0, comment_id.0
            ))
            .map_err(|source| Error::UrlParseError {
                message: format!(
                    "Could not build audio content URL for source id `{}` and comment id `{}`.",
                    source_id.0, comment_id.0,
                ),
                source,
            })
    }

    fn put_emails(&self, bucket_name: &BucketFullName) -> Result<Url> {
        self.base
            .join(&format!("/api/_private/buckets/{}/emails", bucket_name.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build put emails URL for bucket `{}`.",
                    bucket_name,
                ),
            })
    }

    fn dataset_by_id(&self, dataset_id: &DatasetId) -> Result<Url> {
        self.base
            .join(&format!("/api/v1/datasets/id:{}", dataset_id.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build URL for dataset resource with id `{}`.",
                    dataset_id.0
                ),
            })
    }

    fn dataset_by_name(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        self.base
            .join(&format!("/api/v1/datasets/{}", dataset_name.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build URL for dataset resource with name `{}`.",
                    dataset_name.0
                ),
            })
    }

    fn get_labellings(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        self.base
            .join(&format!(
                "/api/_private/datasets/{}/labellings",
                dataset_name.0
            ))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build get labellings URL for dataset `{}`.",
                    dataset_name.0,
                ),
            })
    }

    fn post_labelling(
        &self,
        dataset_name: &DatasetFullName,
        comment_uid: &CommentUid,
    ) -> Result<Url> {
        self.base
            .join(&format!(
                "/api/_private/datasets/{}/labellings/{}",
                dataset_name.0, comment_uid.0,
            ))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build get labellings URL for dataset `{}`.",
                    dataset_name.0,
                ),
            })
    }

    fn bucket_by_id(&self, bucket_id: &BucketId) -> Result<Url> {
        self.base
            .join(&format!("/api/_private/buckets/id:{}", bucket_id.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build URL for bucket resource with id `{}`.",
                    bucket_id.0
                ),
            })
    }

    fn bucket_by_name(&self, bucket_name: &BucketFullName) -> Result<Url> {
        self.base
            .join(&format!("/api/_private/buckets/{}", bucket_name.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build URL for bucket resource with name `{}`.",
                    bucket_name.0
                ),
            })
    }

    fn project_by_name(&self, project_name: &ProjectName) -> Result<Url> {
        self.base
            .join(&format!("/api/_private/projects/{}", project_name.0))
            .map_err(|source| Error::UrlParseError {
                source,
                message: format!(
                    "Could not build URL for project resource with name `{}`.",
                    project_name.0
                ),
            })
    }
}

fn build_http_client(config: &Config) -> Result<HttpClient> {
    let mut builder = HttpClient::builder()
        .gzip(true)
        .danger_accept_invalid_certs(config.accept_invalid_certificates);
    if let Some(proxy) = config.proxy.clone() {
        builder = builder.proxy(Proxy::all(proxy).map_err(Error::BuildHttpClient)?);
    }
    builder.build().map_err(Error::BuildHttpClient)
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
