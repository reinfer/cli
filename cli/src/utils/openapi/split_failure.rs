//! Split-on-failure utility for OpenAPI clients
//!
//! This module provides a reusable abstraction for implementing split-on-failure logic
//! with OpenAPI generated clients, similar to the custom client's `splitable_request` method.
//!
//! When a batch request fails with certain error conditions (422, 400, timeouts), this utility
//! automatically splits the batch into individual requests and retries them, allowing partial
//! success instead of complete failure.

use anyhow::{Context, Result};
use log::debug;
use std::fmt::Debug;

// OpenAPI client types
use openapi::apis::{Error as OpenApiError, ResponseContent};
use openapi::models::{
    sync_comments_response,

    sync_raw_emails_response,
    // Comments API types
    AddCommentsRequest,
    AddCommentsResponse,
    // Emails API types
    AddEmailsToBucketRequest,
    AddEmailsToBucketResponse,
    SyncCommentsRequest,
    SyncCommentsResponse,
    SyncRawEmailsRequest,
    SyncRawEmailsResponse,
};

/// Result of a split-on-failure operation
#[derive(Debug, Clone)]
pub struct SplitOnFailureResult<T> {
    /// The merged response from successful individual requests
    pub response: T,
    /// Number of individual requests that failed
    pub num_failed: usize,
}

/// Trait for requests that can be split into individual requests
pub trait SplittableOpenApiRequest: Clone {
    /// Split the batch request into individual requests
    fn split_request(self) -> Vec<Self>;

    /// Get the count of items in this request
    fn item_count(&self) -> usize;
}

/// Trait for responses that can be merged together
pub trait MergeableOpenApiResponse: Default + Clone {
    /// Merge this response with another response
    fn merge_response(self, other: Self) -> Self;

    /// Create an empty response for use as the starting value
    fn empty_response() -> Self {
        Self::default()
    }
}

/// Execute an OpenAPI call with split-on-failure logic
///
/// # Arguments
/// * `api_call` - A closure that takes a request and returns a Result<Response, Error<ErrorType>>
/// * `request` - The batch request to execute
/// * `operation_name` - A descriptive name for logging
///
/// # Returns
/// * `Ok(SplitOnFailureResult)` - Successful execution with merged results and failure count
/// * `Err` - Unrecoverable error that shouldn't trigger splitting
pub fn execute_with_split_on_failure<RequestT, ResponseT, ErrorT, F>(
    api_call: F,
    request: RequestT,
    operation_name: &str,
) -> Result<SplitOnFailureResult<ResponseT>>
where
    RequestT: SplittableOpenApiRequest,
    ResponseT: MergeableOpenApiResponse,
    ErrorT: Debug,
    F: Fn(RequestT) -> Result<ResponseT, OpenApiError<ErrorT>> + Copy,
{
    debug!(
        "Attempting batch {} with {} items",
        operation_name,
        request.item_count()
    );

    // First, try the entire batch
    match api_call(request.clone()) {
        Ok(response) => {
            debug!("Batch {operation_name} succeeded without splitting");
            Ok(SplitOnFailureResult {
                response,
                num_failed: 0,
            })
        }
        Err(error) if should_split_request(&error) => {
            debug!(
                "Batch {operation_name} failed with splittable error, attempting to split"
            );
            execute_split_requests(api_call, request, operation_name)
        }
        Err(error) => {
            debug!("Batch {operation_name} failed with non-splittable error");
            Err(convert_openapi_error(error)).with_context(|| {
                format!(
                    "Failed to execute {operation_name} (non-splittable error)"
                )
            })
        }
    }
}

/// Execute individual split requests and merge results
fn execute_split_requests<RequestT, ResponseT, ErrorT, F>(
    api_call: F,
    request: RequestT,
    operation_name: &str,
) -> Result<SplitOnFailureResult<ResponseT>>
where
    RequestT: SplittableOpenApiRequest,
    ResponseT: MergeableOpenApiResponse,
    ErrorT: Debug,
    F: Fn(RequestT) -> Result<ResponseT, OpenApiError<ErrorT>>,
{
    let individual_requests = request.split_request();
    let total_requests = individual_requests.len();
    let mut num_failed = 0;
    let mut merged_response = ResponseT::empty_response();

    debug!(
        "Split {operation_name} into {total_requests} individual requests"
    );

    for (index, individual_request) in individual_requests.into_iter().enumerate() {
        match api_call(individual_request) {
            Ok(response) => {
                merged_response = merged_response.merge_response(response);
                debug!(
                    "Individual {} request {}/{} succeeded",
                    operation_name,
                    index + 1,
                    total_requests
                );
            }
            Err(error) => {
                num_failed += 1;
                debug!(
                    "Individual {} request {}/{} failed: {:?}",
                    operation_name,
                    index + 1,
                    total_requests,
                    error
                );
            }
        }
    }

    let num_succeeded = total_requests - num_failed;
    debug!(
        "Split {operation_name} completed: {num_succeeded} succeeded, {num_failed} failed"
    );

    Ok(SplitOnFailureResult {
        response: merged_response,
        num_failed,
    })
}

/// Determine if an OpenAPI error should trigger request splitting
fn should_split_request<T>(error: &OpenApiError<T>) -> bool {
    match error {
        // Network/HTTP-layer errors that might benefit from splitting
        OpenApiError::Reqwest(reqwest_error) => {
            // Split on timeouts - smaller requests might succeed
            reqwest_error.is_timeout()
        }

        // JSON serialization errors - might be caused by payload size
        OpenApiError::Serde(_) => true,

        // File/IO errors - might be transient
        OpenApiError::Io(_) => true,

        // API response errors - check HTTP status codes
        OpenApiError::ResponseError(ResponseContent { status, .. }) => {
            // These are the same conditions as the custom client
            *status == reqwest::StatusCode::UNPROCESSABLE_ENTITY  // 422 - validation errors
                || *status == reqwest::StatusCode::BAD_REQUEST // 400 - malformed request
        }
    }
}

/// Convert OpenAPI error to anyhow error
fn convert_openapi_error<T: Debug>(error: OpenApiError<T>) -> anyhow::Error {
    match error {
        OpenApiError::Reqwest(e) => anyhow::Error::new(e).context("HTTP request error"),
        OpenApiError::Serde(e) => anyhow::Error::new(e).context("JSON serialization error"),
        OpenApiError::Io(e) => anyhow::Error::new(e).context("I/O error"),
        OpenApiError::ResponseError(ResponseContent {
            status, content, ..
        }) => {
            anyhow::anyhow!("error in response: status code {}: {}", status, content)
        }
    }
}

// =============================================================================
// Trait implementations for common OpenAPI request/response types
// =============================================================================

// =============================================================================
// Comments API implementations
// =============================================================================
impl SplittableOpenApiRequest for AddCommentsRequest {
    fn split_request(self) -> Vec<Self> {
        self.comments
            .into_iter()
            .map(|comment| AddCommentsRequest::new(vec![comment]))
            .collect()
    }

    fn item_count(&self) -> usize {
        self.comments.len()
    }
}

impl SplittableOpenApiRequest for SyncCommentsRequest {
    fn split_request(self) -> Vec<Self> {
        self.comments
            .into_iter()
            .map(|comment| SyncCommentsRequest::new(vec![comment]))
            .collect()
    }

    fn item_count(&self) -> usize {
        self.comments.len()
    }
}

impl MergeableOpenApiResponse for AddCommentsResponse {
    fn merge_response(self, _other: Self) -> Self {
        // AddCommentsResponse only contains status information, no numerical data to merge
        self
    }
}

impl MergeableOpenApiResponse for SyncCommentsResponse {
    fn merge_response(self, other: Self) -> Self {
        SyncCommentsResponse::new(
            self.status, // Status should be the same for all successful responses
            self.updated + other.updated,
            self.updated_properties_only + other.updated_properties_only,
            self.updated_text_changed + other.updated_text_changed,
            self.unchanged + other.unchanged,
            self.new + other.new,
        )
    }

    fn empty_response() -> Self {
        SyncCommentsResponse::new(
            sync_comments_response::Status::Ok,
            0, // updated
            0, // updated_properties_only
            0, // updated_text_changed
            0, // unchanged
            0, // new
        )
    }
}

// =============================================================================
// Emails API implementations
// =============================================================================
impl SplittableOpenApiRequest for AddEmailsToBucketRequest {
    fn split_request(self) -> Vec<Self> {
        self.emails
            .into_iter()
            .map(|email| AddEmailsToBucketRequest::new(vec![email]))
            .collect()
    }

    fn item_count(&self) -> usize {
        self.emails.len()
    }
}

impl MergeableOpenApiResponse for AddEmailsToBucketResponse {
    fn merge_response(self, _other: Self) -> Self {
        // AddEmailsToBucketResponse only contains status information, no numerical data to merge
        self
    }
}

// =============================================================================
// Raw emails API implementations
// =============================================================================
impl SplittableOpenApiRequest for SyncRawEmailsRequest {
    fn split_request(self) -> Vec<Self> {
        self.documents
            .into_iter()
            .map(|document| {
                let mut request = SyncRawEmailsRequest::new(vec![document]);
                request.include_comments = self.include_comments;
                request.transform_tag = self.transform_tag.clone();
                request.override_user_properties = self.override_user_properties.clone();
                request
            })
            .collect()
    }

    fn item_count(&self) -> usize {
        self.documents.len()
    }
}

impl MergeableOpenApiResponse for SyncRawEmailsResponse {
    fn merge_response(self, other: Self) -> Self {
        // Note: SyncRawEmailsResponse has additional fields (updated_properties_only, updated_text_changed)
        // that aren't available in this merge context, so we set them to 0
        SyncRawEmailsResponse::new(
            self.status, // Status should be the same for all successful responses
            self.updated + other.updated,
            0, // updated_properties_only - not tracked during split operations
            0, // updated_text_changed - not tracked during split operations
            self.unchanged + other.unchanged,
            self.new + other.new,
        )
    }

    fn empty_response() -> Self {
        SyncRawEmailsResponse::new(
            sync_raw_emails_response::Status::Ok,
            0, // updated
            0, // updated_properties_only
            0, // updated_text_changed
            0, // unchanged
            0, // new
        )
    }
}
