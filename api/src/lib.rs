#![deny(clippy::all)]
mod error;
pub mod resources;
pub mod retry;

use chrono::{DateTime, Utc};
use http::Method;
use log::debug;
use once_cell::sync::Lazy;
use reqwest::{
    blocking::{multipart::Form, Client as HttpClient, Response as HttpResponse},
    header::{self, HeaderMap, HeaderValue},
    IntoUrl, Proxy, Result as ReqwestResult,
};
use resources::{
    attachments::UploadAttachmentResponse,
    bucket_statistics::GetBucketStatisticsResponse,
    comment::{AttachmentReference, CommentTimestampFilter},
    dataset::{
        QueryRequestParams, QueryResponse,
        StatisticsRequestParams as DatasetStatisticsRequestParams, SummaryRequestParams,
        SummaryResponse,
    },
    documents::{Document, SyncRawEmailsRequest, SyncRawEmailsResponse},
    email::{Email, GetEmailResponse},
    integration::{
        GetIntegrationResponse, GetIntegrationsResponse, Integration, NewIntegration,
        PostIntegrationRequest, PostIntegrationResponse, PutIntegrationRequest,
        PutIntegrationResponse,
    },
    project::ForceDeleteProject,
    quota::{GetQuotasResponse, Quota},
    source::StatisticsRequestParams as SourceStatisticsRequestParams,
    stream::{GetStreamResponse, NewStream, PutStreamRequest, PutStreamResponse},
    validation::{
        LabelValidation, LabelValidationRequest, LabelValidationResponse, ValidationResponse,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    cell::Cell,
    fmt::{Debug, Display},
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
use url::Url;

use crate::resources::{
    audit::{AuditQueryFilter, AuditQueryRequest, AuditQueryResponse},
    bucket::{
        CreateRequest as CreateBucketRequest, CreateResponse as CreateBucketResponse,
        GetAvailableResponse as GetAvailableBucketsResponse, GetResponse as GetBucketResponse,
    },
    bucket_statistics::Statistics as BucketStatistics,
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
    quota::{CreateQuota, TenantQuotaKind},
    source::{
        CreateRequest as CreateSourceRequest, CreateResponse as CreateSourceResponse,
        GetAvailableResponse as GetAvailableSourcesResponse, GetResponse as GetSourceResponse,
        UpdateRequest as UpdateSourceRequest, UpdateResponse as UpdateSourceResponse,
    },
    statistics::GetResponse as GetStatisticsResponse,
    stream::{
        AdvanceRequest as StreamAdvanceRequest, FetchRequest as StreamFetchRequest,
        GetStreamsResponse, ResetRequest as StreamResetRequest,
        TagExceptionsRequest as TagStreamExceptionsRequest,
    },
    tenant_id::TenantId,
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
            Identifier as BucketIdentifier, Name as BucketName, NewBucket,
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
        email::{
            Continuation as EmailContinuation, EmailsIterPage, Id as EmailId, Mailbox, MimeContent,
            NewEmail,
        },
        entity_def::{EntityDef, Id as EntityDefId, Name as EntityName, NewEntityDef},
        integration::FullName as IntegrationFullName,
        label_def::{
            LabelDef, LabelDefPretrained, MoonFormFieldDef, Name as LabelName, NewLabelDef,
            NewLabelDefPretrained, PretrainedId as LabelDefPretrainedId,
        },
        label_group::{
            LabelGroup, Name as LabelGroupName, NewLabelGroup, DEFAULT_LABEL_GROUP_NAME,
        },
        project::{NewProject, Project, ProjectName, UpdateProject},
        source::{
            FullName as SourceFullName, Id as SourceId, Identifier as SourceIdentifier,
            Name as SourceName, NewSource, Source, SourceKind, TransformTag, UpdateSource,
        },
        statistics::Statistics as CommentStatistics,
        stream::{
            Batch as StreamBatch, FullName as StreamFullName, SequenceId as StreamSequenceId,
            Stream, StreamException, StreamExceptionMetadata,
        },
        user::{
            Email as UserEmail, GlobalPermission, Id as UserId, Identifier as UserIdentifier,
            ModifiedPermissions, NewUser, ProjectPermission, UpdateUser, User, Username,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token(pub String);

pub trait SplittableRequest {
    fn split(self) -> impl Iterator<Item = Self>
    where
        Self: Sized;

    fn count(&self) -> usize;
}

pub struct SplitableRequestResponse<ResponseT>
where
    for<'de> ResponseT: Deserialize<'de> + ReducibleResponse,
{
    pub response: ResponseT,
    pub num_failed: usize,
}

pub trait ReducibleResponse {
    fn merge(self, _b: Self) -> Self
    where
        Self: std::default::Default,
    {
        Default::default()
    }

    fn empty() -> Self
    where
        Self: std::default::Default,
    {
        Default::default()
    }
}

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
    pub include_markup: bool,
}

#[derive(Serialize)]
pub struct GetEmailsIterPageQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<&'a EmailContinuation>,
    pub limit: usize,
}

#[derive(Serialize)]
pub struct GetCommentQuery {
    pub include_markup: bool,
}

#[derive(Serialize)]
pub struct GetEmailQuery {
    pub id: String,
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

    /// Get the base url for the client
    pub fn base_url(&self) -> &Url {
        &self.endpoints.base
    }

    /// List all visible sources.
    pub fn get_sources(&self) -> Result<Vec<Source>> {
        Ok(self
            .get::<_, GetAvailableSourcesResponse>(self.endpoints.sources.clone())?
            .sources)
    }

    /// Get a source by either id or name.
    pub fn get_user(&self, user: impl Into<UserIdentifier>) -> Result<User> {
        Ok(match user.into() {
            UserIdentifier::Id(user_id) => {
                self.get::<_, GetUserResponse>(self.endpoints.user_by_id(&user_id)?)?
                    .user
            }
        })
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
        self.delete(self.endpoints.source_by_id(&source_id)?)
    }

    /// Set a quota
    pub fn create_quota(
        &self,
        target_tenant_id: &TenantId,
        tenant_quota_kind: TenantQuotaKind,
        options: CreateQuota,
    ) -> Result<()> {
        self.post(
            self.endpoints.quota(target_tenant_id, tenant_quota_kind)?,
            options,
            Retry::Yes,
        )
    }

    /// Get quotas for current tenant
    pub fn get_quotas(&self) -> Result<Vec<Quota>> {
        Ok(self
            .get::<_, GetQuotasResponse>(self.endpoints.quotas()?)?
            .quotas)
    }

    /// Delete a user.
    pub fn delete_user(&self, user: impl Into<UserIdentifier>) -> Result<()> {
        let UserIdentifier::Id(user_id) = user.into();
        self.delete(self.endpoints.user_by_id(&user_id)?)
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
        self.delete_query(
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
            include_markup: true,
        };
        self.get_query(self.endpoints.comments(source_name)?, Some(&query_params))
    }

    /// Iterate through all comments for a given dataset query.
    pub fn get_dataset_query_iter<'a>(
        &'a self,
        dataset_name: &'a DatasetFullName,
        params: &'a mut QueryRequestParams,
    ) -> DatasetQueryIter<'a> {
        DatasetQueryIter::new(self, dataset_name, params)
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

    /// Get a single of email from a bucket.
    pub fn get_email(&self, bucket_name: &BucketFullName, id: EmailId) -> Result<Vec<Email>> {
        let query_params = GetEmailQuery { id: id.0 };
        Ok(self
            .get_query::<_, _, GetEmailResponse>(
                self.endpoints.get_emails(bucket_name)?,
                Some(&query_params),
            )?
            .emails)
    }

    /// Get a page of emails from a bucket.
    pub fn get_emails_iter_page(
        &self,
        bucket_name: &BucketFullName,
        continuation: Option<&EmailContinuation>,
        limit: usize,
    ) -> Result<EmailsIterPage> {
        let query_params = GetEmailsIterPageQuery {
            continuation,
            limit,
        };
        self.post(
            self.endpoints.get_emails(bucket_name)?,
            Some(&query_params),
            Retry::Yes,
        )
    }

    /// Iterate through all comments in a source.
    pub fn get_emails_iter<'a>(
        &'a self,
        bucket_name: &'a BucketFullName,
        page_size: Option<usize>,
    ) -> EmailsIter<'a> {
        EmailsIter::new(self, bucket_name, page_size)
    }

    /// Get a single comment by id.
    pub fn get_comment<'a>(
        &'a self,
        source_name: &'a SourceFullName,
        comment_id: &'a CommentId,
    ) -> Result<Comment> {
        let query_params = GetCommentQuery {
            include_markup: true,
        };
        Ok(self
            .get_query::<_, _, GetCommentResponse>(
                self.endpoints.comment_by_id(source_name, comment_id)?,
                Some(&query_params),
            )?
            .comment)
    }
    pub fn post_integration(
        &self,
        name: &IntegrationFullName,
        integration: &NewIntegration,
    ) -> Result<PostIntegrationResponse> {
        self.request(
            &Method::POST,
            &self.endpoints.integration(name)?,
            &Some(PostIntegrationRequest {
                integration: integration.clone(),
            }),
            &None::<()>,
            &Retry::No,
        )
    }

    pub fn put_integration(
        &self,
        name: &IntegrationFullName,
        integration: &NewIntegration,
    ) -> Result<PutIntegrationResponse> {
        self.request(
            &Method::PUT,
            &self.endpoints.integration(name)?,
            &Some(PutIntegrationRequest {
                integration: integration.clone(),
            }),
            &None::<()>,
            &Retry::No,
        )
    }

    pub fn put_comments_split_on_failure(
        &self,
        source_name: &SourceFullName,
        comments: Vec<NewComment>,
        no_charge: bool,
    ) -> Result<SplitableRequestResponse<PutCommentsResponse>> {
        // Retrying here despite the potential for 409's in order to increase reliability when
        // working with poor connection

        self.splitable_request(
            Method::PUT,
            self.endpoints.put_comments(source_name)?,
            PutCommentsRequest { comments },
            Some(NoChargeQuery { no_charge }),
            Retry::Yes,
        )
    }

    pub fn put_comments(
        &self,
        source_name: &SourceFullName,
        comments: Vec<NewComment>,
        no_charge: bool,
    ) -> Result<PutCommentsResponse> {
        // Retrying here despite the potential for 409's in order to increase reliability when
        // working with poor connection
        self.request(
            &Method::PUT,
            &self.endpoints.put_comments(source_name)?,
            &Some(PutCommentsRequest { comments }),
            &Some(NoChargeQuery { no_charge }),
            &Retry::Yes,
        )
    }

    pub fn put_stream(
        &self,
        dataset_name: &DatasetFullName,
        stream: &NewStream,
    ) -> Result<PutStreamResponse> {
        self.put(
            self.endpoints.streams(dataset_name)?,
            Some(PutStreamRequest { stream }),
        )
    }

    pub fn get_audit_events(
        &self,
        minimum_timestamp: Option<DateTime<Utc>>,
        maximum_timestamp: Option<DateTime<Utc>>,
        continuation: Option<Continuation>,
    ) -> Result<AuditQueryResponse> {
        self.post::<_, _, AuditQueryResponse>(
            self.endpoints.audit_events_query()?,
            AuditQueryRequest {
                continuation,
                filter: AuditQueryFilter {
                    timestamp: CommentTimestampFilter {
                        minimum: minimum_timestamp,
                        maximum: maximum_timestamp,
                    },
                },
            },
            Retry::Yes,
        )
    }

    pub fn get_validation(
        &self,
        dataset_name: &DatasetFullName,
        model_version: &ModelVersion,
    ) -> Result<ValidationResponse> {
        self.get::<_, ValidationResponse>(self.endpoints.validation(dataset_name, model_version)?)
    }

    pub fn get_label_validation(
        &self,
        label: &LabelName,
        dataset_name: &DatasetFullName,
        model_version: &ModelVersion,
    ) -> Result<LabelValidation> {
        Ok(self
            .post::<_, _, LabelValidationResponse>(
                self.endpoints
                    .label_validation(dataset_name, model_version)?,
                LabelValidationRequest {
                    label: label.clone(),
                },
                Retry::Yes,
            )?
            .label_validation)
    }

    pub fn sync_comments(
        &self,
        source_name: &SourceFullName,
        comments: Vec<NewComment>,
        no_charge: bool,
    ) -> Result<SyncCommentsResponse> {
        self.request(
            &Method::POST,
            &self.endpoints.sync_comments(source_name)?,
            &Some(SyncCommentsRequest { comments }),
            &Some(NoChargeQuery { no_charge }),
            &Retry::Yes,
        )
    }

    pub fn sync_comments_split_on_failure(
        &self,
        source_name: &SourceFullName,
        comments: Vec<NewComment>,
        no_charge: bool,
    ) -> Result<SplitableRequestResponse<SyncCommentsResponse>> {
        self.splitable_request(
            Method::POST,
            self.endpoints.sync_comments(source_name)?,
            SyncCommentsRequest { comments },
            Some(NoChargeQuery { no_charge }),
            Retry::Yes,
        )
    }

    pub fn sync_raw_emails(
        &self,
        source_name: &SourceFullName,
        documents: &[Document],
        transform_tag: &TransformTag,
        include_comments: bool,
        no_charge: bool,
    ) -> Result<SyncRawEmailsResponse> {
        self.request(
            &Method::POST,
            &self.endpoints.sync_comments_raw_emails(source_name)?,
            &Some(SyncRawEmailsRequest {
                documents,
                transform_tag,
                include_comments,
            }),
            &Some(NoChargeQuery { no_charge }),
            &Retry::Yes,
        )
    }

    pub fn put_emails_split_on_failure(
        &self,
        bucket_name: &BucketFullName,
        emails: Vec<NewEmail>,
        no_charge: bool,
    ) -> Result<SplitableRequestResponse<PutEmailsResponse>> {
        self.splitable_request(
            Method::PUT,
            self.endpoints.put_emails(bucket_name)?,
            PutEmailsRequest { emails },
            Some(NoChargeQuery { no_charge }),
            Retry::Yes,
        )
    }

    pub fn put_emails(
        &self,
        bucket_name: &BucketFullName,
        emails: Vec<NewEmail>,
        no_charge: bool,
    ) -> Result<PutEmailsResponse> {
        self.request(
            &Method::PUT,
            &self.endpoints.put_emails(bucket_name)?,
            &Some(PutEmailsRequest { emails }),
            &Some(NoChargeQuery { no_charge }),
            &Retry::Yes,
        )
    }

    pub fn post_user(&self, user_id: &UserId, user: UpdateUser) -> Result<PostUserResponse> {
        self.post(
            self.endpoints.post_user(user_id)?,
            PostUserRequest { user: &user },
            Retry::Yes,
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

    pub fn upload_comment_attachment(
        &self,
        source_id: &SourceId,
        comment_id: &CommentId,
        attachment_index: usize,
        attachment: &PathBuf,
    ) -> Result<UploadAttachmentResponse> {
        let url = self
            .endpoints
            .attachment_upload(source_id, comment_id, attachment_index)?;

        if !attachment.is_file() || !attachment.exists() {
            return Err(Error::FileDoesNotExist {
                path: attachment.clone(),
            });
        }

        let do_request = || {
            let form = Form::new()
                .file("file", attachment)
                .map_err(|source| Error::Unknown {
                    message: "PUT comment attachment operation failed".to_owned(),
                    source: source.into(),
                })
                .unwrap();
            let request = self
                .http_client
                .request(Method::PUT, url.clone())
                .multipart(form)
                .headers(self.headers.clone());

            request.send()
        };

        let result = self.with_retries(do_request);

        let http_response = result.map_err(|source| Error::ReqwestError {
            source,
            message: "Operation failed.".to_string(),
        })?;

        let status = http_response.status();

        http_response
            .json::<Response<UploadAttachmentResponse>>()
            .map_err(Error::BadJsonResponse)?
            .into_result(status)
    }

    pub fn get_attachment(&self, reference: &AttachmentReference) -> Result<Vec<u8>> {
        let mut response = self.raw_request(
            &Method::GET,
            &self.endpoints.attachment_reference(reference)?,
            &None::<()>,
            &None::<()>,
            &Retry::Yes,
        )?;

        let mut buffer = Vec::new();

        response
            .read_to_end(&mut buffer)
            .map_err(|source| Error::Unknown {
                message: "Failed to read buffer".to_string(),
                source: Box::new(source),
            })?;

        Ok(buffer)
    }

    pub fn get_integrations(&self) -> Result<Vec<Integration>> {
        Ok(self
            .get::<_, GetIntegrationsResponse>(self.endpoints.integrations()?)?
            .integrations)
    }

    pub fn get_integration(&self, name: &IntegrationFullName) -> Result<Integration> {
        Ok(self
            .get::<_, GetIntegrationResponse>(self.endpoints.integration(name)?)?
            .integration)
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
        self.delete(self.endpoints.dataset_by_id(&dataset_id)?)
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
    }

    /// Get predictions for a given a dataset, a model version, and a list of comment UIDs.
    pub fn get_comment_predictions<'a>(
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
            )?
            .predictions)
    }

    pub fn get_streams(&self, dataset_name: &DatasetFullName) -> Result<Vec<Stream>> {
        Ok(self
            .get::<_, GetStreamsResponse>(self.endpoints.streams(dataset_name)?)?
            .streams)
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

    pub fn dataset_summary(
        &self,
        dataset_name: &DatasetFullName,
        params: &SummaryRequestParams,
    ) -> Result<SummaryResponse> {
        self.post::<_, _, SummaryResponse>(
            self.endpoints.dataset_summary(dataset_name)?,
            serde_json::to_value(params).expect("summary params serialization error"),
            Retry::Yes,
        )
    }

    pub fn query_dataset(
        &self,
        dataset_name: &DatasetFullName,
        params: &QueryRequestParams,
    ) -> Result<QueryResponse> {
        self.post::<_, _, QueryResponse>(
            self.endpoints.query_dataset(dataset_name)?,
            serde_json::to_value(params).expect("query params serialization error"),
            Retry::Yes,
        )
    }

    pub fn send_welcome_email(&self, user_id: UserId) -> Result<()> {
        self.post::<_, _, WelcomeEmailResponse>(
            self.endpoints.welcome_email(&user_id)?,
            json!({}),
            Retry::No,
        )?;
        Ok(())
    }

    pub fn get_bucket_statistics(&self, bucket_name: &BucketFullName) -> Result<BucketStatistics> {
        Ok(self
            .get::<_, GetBucketStatisticsResponse>(self.endpoints.bucket_statistics(bucket_name)?)?
            .statistics)
    }

    pub fn get_dataset_statistics(
        &self,
        dataset_name: &DatasetFullName,
        params: &DatasetStatisticsRequestParams,
    ) -> Result<CommentStatistics> {
        Ok(self
            .post::<_, _, GetStatisticsResponse>(
                self.endpoints.dataset_statistics(dataset_name)?,
                serde_json::to_value(params)
                    .expect("dataset statistics params serialization error"),
                Retry::No,
            )?
            .statistics)
    }

    pub fn get_source_statistics(
        &self,
        source_name: &SourceFullName,
        params: &SourceStatisticsRequestParams,
    ) -> Result<CommentStatistics> {
        Ok(self
            .post::<_, _, GetStatisticsResponse>(
                self.endpoints.source_statistics(source_name)?,
                serde_json::to_value(params).expect("source statistics params serialization error"),
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
        self.delete(self.endpoints.bucket_by_id(&bucket_id)?)
    }

    pub fn fetch_stream_comments(
        &self,
        stream_name: &StreamFullName,
        size: u32,
    ) -> Result<StreamBatch> {
        self.post(
            self.endpoints.stream_fetch(stream_name)?,
            StreamFetchRequest { size },
            Retry::No,
        )
    }

    pub fn get_stream(&self, stream_name: &StreamFullName) -> Result<Stream> {
        Ok(self
            .get::<_, GetStreamResponse>(self.endpoints.stream(stream_name)?)?
            .stream)
    }

    pub fn advance_stream(
        &self,
        stream_name: &StreamFullName,
        sequence_id: StreamSequenceId,
    ) -> Result<()> {
        self.post::<_, _, serde::de::IgnoredAny>(
            self.endpoints.stream_advance(stream_name)?,
            StreamAdvanceRequest { sequence_id },
            Retry::No,
        )?;
        Ok(())
    }

    pub fn reset_stream(
        &self,
        stream_name: &StreamFullName,
        to_comment_created_at: DateTime<Utc>,
    ) -> Result<()> {
        self.post::<_, _, serde::de::IgnoredAny>(
            self.endpoints.stream_reset(stream_name)?,
            StreamResetRequest {
                to_comment_created_at,
            },
            Retry::No,
        )?;
        Ok(())
    }

    pub fn tag_stream_exceptions(
        &self,
        stream_name: &StreamFullName,
        exceptions: &[StreamException],
    ) -> Result<()> {
        self.put::<_, _, serde::de::IgnoredAny>(
            self.endpoints.stream_exceptions(stream_name)?,
            TagStreamExceptionsRequest { exceptions },
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
        self.request(&Method::GET, &url, &None::<()>, &None::<()>, &Retry::Yes)
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
        self.request(&Method::GET, &url, &None::<()>, &Some(query), &Retry::Yes)
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
        self.request(&Method::POST, &url, &Some(request), &None::<()>, &retry)
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
        self.request(&Method::PUT, &url, &Some(request), &None::<()>, &Retry::Yes)
    }

    fn raw_request<LocationT, RequestT, QueryT>(
        &self,
        method: &Method,
        url: &LocationT,
        body: &Option<RequestT>,
        query: &Option<QueryT>,
        retry: &Retry,
    ) -> Result<reqwest::blocking::Response>
    where
        LocationT: IntoUrl + Display + Clone,
        RequestT: Serialize,
        QueryT: Serialize,
    {
        let do_request = || {
            let request = self
                .http_client
                .request(method.clone(), url.clone())
                .headers(self.headers.clone());
            let request = match &query {
                Some(query) => request.query(query),
                None => request,
            };
            let request = match &body {
                Some(body) => request.json(body),
                None => request,
            };
            request.send()
        };

        let result = match retry {
            Retry::Yes => self.with_retries(do_request),
            Retry::No => do_request(),
        };
        let http_response = result.map_err(|source| Error::ReqwestError {
            source,
            message: format!("{method} operation failed."),
        })?;

        Ok(http_response)
    }

    fn splitable_request<LocationT, RequestT, SuccessT, QueryT>(
        &self,
        method: Method,
        url: LocationT,
        body: RequestT,
        query: Option<QueryT>,
        retry: Retry,
    ) -> Result<SplitableRequestResponse<SuccessT>>
    where
        LocationT: IntoUrl + Display + Clone,
        RequestT: Serialize + SplittableRequest + Clone,
        QueryT: Serialize + Clone,
        for<'de> SuccessT: Deserialize<'de> + ReducibleResponse + Clone + Default,
    {
        debug!("Attempting {method} `{url}`");
        let result: Result<SuccessT> =
            self.request(&method, &url, &Some(body.clone()), &query, &retry);

        fn should_split(error: &Error) -> bool {
            if let Error::Api { status_code, .. } = error {
                *status_code == reqwest::StatusCode::UNPROCESSABLE_ENTITY
                    || *status_code == reqwest::StatusCode::BAD_REQUEST
            } else {
                false
            }
        }

        match result {
            Ok(response) => Ok(SplitableRequestResponse {
                response,
                num_failed: 0,
            }),
            Err(error) if should_split(&error) => {
                let mut num_failed = 0;
                let response = body
                    .split()
                    .filter_map(|request| {
                        match self.request(&method, &url, &Some(request), &query, &retry) {
                            Ok(response) => Some(response),
                            Err(_) => {
                                num_failed += 1;
                                None
                            }
                        }
                    })
                    .fold(SuccessT::empty(), |merged, next: SuccessT| {
                        merged.merge(next)
                    });

                Ok(SplitableRequestResponse {
                    num_failed,
                    response,
                })
            }
            Err(error) => Err(error),
        }
    }

    fn request<LocationT, RequestT, SuccessT, QueryT>(
        &self,
        method: &Method,
        url: &LocationT,
        body: &Option<RequestT>,
        query: &Option<QueryT>,
        retry: &Retry,
    ) -> Result<SuccessT>
    where
        LocationT: IntoUrl + Display + Clone,
        RequestT: Serialize,
        QueryT: Serialize + Clone,
        for<'de> SuccessT: Deserialize<'de>,
    {
        debug!("Attempting {} `{}`", method, url);
        let http_response = self.raw_request(method, url, body, query, retry)?;

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

#[derive(Copy, Clone)]
enum Retry {
    Yes,
    No,
}

pub struct DatasetQueryIter<'a> {
    client: &'a Client,
    dataset_name: &'a DatasetFullName,
    done: bool,
    params: &'a mut QueryRequestParams,
}

impl<'a> DatasetQueryIter<'a> {
    fn new(
        client: &'a Client,
        dataset_name: &'a DatasetFullName,
        params: &'a mut QueryRequestParams,
    ) -> Self {
        Self {
            client,
            dataset_name,
            done: false,
            params,
        }
    }
}

impl<'a> Iterator for DatasetQueryIter<'a> {
    type Item = Result<Vec<AnnotatedComment>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let response = self.client.query_dataset(self.dataset_name, self.params);
        Some(response.map(|page| {
            self.params.continuation = page.continuation;
            self.done = self.params.continuation.is_none();
            page.results
        }))
    }
}

pub enum ContinuationKind {
    Timestamp(DateTime<Utc>),
    Continuation(Continuation),
}

pub struct EmailsIter<'a> {
    client: &'a Client,
    bucket_name: &'a BucketFullName,
    continuation: Option<EmailContinuation>,
    done: bool,
    page_size: usize,
}

impl<'a> EmailsIter<'a> {
    // Default number of emails per page to request from API.
    pub const DEFAULT_PAGE_SIZE: usize = 64;
    // Maximum number of emails per page which can be requested from the API.
    pub const MAX_PAGE_SIZE: usize = 256;

    fn new(client: &'a Client, bucket_name: &'a BucketFullName, page_size: Option<usize>) -> Self {
        Self {
            client,
            bucket_name,
            continuation: None,
            done: false,
            page_size: page_size.unwrap_or(Self::DEFAULT_PAGE_SIZE),
        }
    }
}

impl<'a> Iterator for EmailsIter<'a> {
    type Item = Result<Vec<Email>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let response = self.client.get_emails_iter_page(
            self.bucket_name,
            self.continuation.as_ref(),
            self.page_size,
        );
        Some(response.map(|page| {
            self.continuation = page.continuation;
            self.done = self.continuation.is_none();
            page.emails
        }))
    }
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

#[derive(Debug, Serialize, Clone, Copy)]
struct NoChargeQuery {
    no_charge: bool,
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

    fn audit_events_query(&self) -> Result<Url> {
        construct_endpoint(&self.base, &["api", "v1", "audit_events", "query"])
    }

    fn integrations(&self) -> Result<Url> {
        construct_endpoint(&self.base, &["api", "_private", "integrations"])
    }

    fn integration(&self, name: &IntegrationFullName) -> Result<Url> {
        construct_endpoint(&self.base, &["api", "_private", "integrations", &name.0])
    }

    fn attachment_reference(&self, reference: &AttachmentReference) -> Result<Url> {
        construct_endpoint(&self.base, &["api", "v1", "attachments", &reference.0])
    }

    fn attachment_upload(
        &self,
        source_id: &SourceId,
        comment_id: &CommentId,
        attachment_index: usize,
    ) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "_private",
                "sources",
                &format!("id:{}", source_id.0),
                "comments",
                &comment_id.0,
                "attachments",
                &attachment_index.to_string(),
            ],
        )
    }

    fn validation(
        &self,
        dataset_name: &DatasetFullName,
        model_version: &ModelVersion,
    ) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "_private",
                "datasets",
                &dataset_name.0,
                "labellers",
                &model_version.0.to_string(),
                "validation",
            ],
        )
    }

    fn label_validation(
        &self,
        dataset_name: &DatasetFullName,
        model_version: &ModelVersion,
    ) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "_private",
                "datasets",
                &dataset_name.0,
                "labellers",
                &model_version.0.to_string(),
                "label-validation",
            ],
        )
    }
    fn bucket_statistics(&self, bucket_name: &BucketFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "buckets", &bucket_name.0, "statistics"],
        )
    }

    fn dataset_summary(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "datasets", &dataset_name.0, "summary"],
        )
    }

    fn query_dataset(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "datasets", &dataset_name.0, "query"],
        )
    }

    fn streams(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "v1", "datasets", &dataset_name.0, "streams"],
        )
    }

    fn stream(&self, stream_name: &StreamFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "datasets",
                &stream_name.dataset.0,
                "streams",
                &stream_name.stream.0,
            ],
        )
    }

    fn stream_fetch(&self, stream_name: &StreamFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "datasets",
                &stream_name.dataset.0,
                "streams",
                &stream_name.stream.0,
                "fetch",
            ],
        )
    }

    fn stream_advance(&self, stream_name: &StreamFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "datasets",
                &stream_name.dataset.0,
                "streams",
                &stream_name.stream.0,
                "advance",
            ],
        )
    }

    fn stream_reset(&self, stream_name: &StreamFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "datasets",
                &stream_name.dataset.0,
                "streams",
                &stream_name.stream.0,
                "reset",
            ],
        )
    }

    fn stream_exceptions(&self, stream_name: &StreamFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "v1",
                "datasets",
                &stream_name.dataset.0,
                "streams",
                &stream_name.stream.0,
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

    fn dataset_statistics(&self, dataset_name: &DatasetFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "datasets", &dataset_name.0, "statistics"],
        )
    }

    fn source_statistics(&self, source_name: &SourceFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "v1", "sources", &source_name.0, "statistics"],
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

    fn quotas(&self) -> Result<Url> {
        construct_endpoint(&self.base, &["api", "_private", "quotas"])
    }

    fn quota(&self, tenant_id: &TenantId, tenant_quota_kind: TenantQuotaKind) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &[
                "api",
                "_private",
                "quotas",
                &tenant_id.to_string(),
                &tenant_quota_kind.to_string(),
            ],
        )
    }

    fn put_comments(&self, source_name: &SourceFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "sources", &source_name.0, "comments"],
        )
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

    fn sync_comments_raw_emails(&self, source_name: &SourceFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "v1", "sources", &source_name.0, "sync-raw-emails"],
        )
    }

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

    fn get_emails(&self, bucket_name: &BucketFullName) -> Result<Url> {
        construct_endpoint(
            &self.base,
            &["api", "_private", "buckets", &bucket_name.0, "emails"],
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

const DEFAULT_HTTP_TIMEOUT_SECONDS: u64 = 120;

fn build_http_client(config: &Config) -> Result<HttpClient> {
    let mut builder = HttpClient::builder()
        .gzip(true)
        .danger_accept_invalid_certs(config.accept_invalid_certificates)
        .timeout(Some(Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECONDS)));

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
    fn test_construct_endpoint() {
        let url = construct_endpoint(
            &Url::parse("https://cloud.uipath.com/org/tenant/reinfer_").unwrap(),
            &["api", "v1", "sources", "project", "source", "sync"],
        )
        .unwrap();

        assert_eq!(
            url.to_string(),
            "https://cloud.uipath.com/org/tenant/reinfer_/api/v1/sources/project/source/sync"
        )
    }

    #[test]
    fn test_id_list_query() {
        assert_eq!(id_list_query(Vec::new().iter()), Vec::new());
        assert_eq!(
            id_list_query(["foo".to_owned()].iter()),
            vec![("id", "foo")]
        );
        assert_eq!(
            id_list_query(
                [
                    "Stream".to_owned(),
                    "River".to_owned(),
                    "Waterfall".to_owned()
                ]
                .iter()
            ),
            [("id", "Stream"), ("id", "River"), ("id", "Waterfall"),]
        );
    }
}
