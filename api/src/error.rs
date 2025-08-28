use std::path::PathBuf;

use reqwest::StatusCode;

use openapi::apis::Error as OAError;
use openapi::apis::ResponseContent;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API request failed with {}: {}", status_code, message)]
    Api {
        status_code: StatusCode,
        message: String,
    },

    #[error("Invalid endpoint: '{}'", endpoint)]
    BadEndpoint { endpoint: url::Url },

    #[error("Bad token: {}", token)]
    BadToken { token: String },

    #[error("File does not exist : {}", path.to_string_lossy())]
    FileDoesNotExist { path: PathBuf },

    #[error("Expected <owner>/<name> or a source id, got: {}", identifier)]
    BadSourceIdentifier { identifier: String },

    #[error("Expected <owner>/<name> or a dataset id, got: {}", identifier)]
    BadDatasetIdentifier { identifier: String },

    #[error("Expected <owner>/<name>: {}", identifier)]
    BadIntegrationIdentifier { identifier: String },

    #[error("Expected <owner>/<dataset>/<stream>: {}", identifier)]
    BadStreamName { identifier: String },

    #[error("Expected u64: {}", version)]
    BadStreamModelVersion { version: String },

    #[error(
        "Expected a user id (usernames and emails are not supported), got: {}",
        identifier
    )]
    BadUserIdentifier { identifier: String },

    #[error("Expected a valid project name, got: {}", identifier)]
    BadProjectIdentifier { identifier: String },

    #[error("Unknown project permission: {}", permission)]
    BadProjectPermission { permission: String },

    #[error("Unknown global permission: {}", permission)]
    BadGlobalPermission { permission: String },

    #[error("Expected <owner>/<name> or a bucket id, got: {}", identifier)]
    BadBucketIdentifier { identifier: String },

    #[error("Expected <owner>/<name>, got: {}", name)]
    BadBucketName { name: String },

    #[error("Expected a valid bucket type, got: {}", bucket_type)]
    BadBucketType { bucket_type: String },

    #[error("Expected a valid quota kind, got: {}", tenant_quota_kind)]
    BadTenantQuotaKind { tenant_quota_kind: String },

    #[error("Could not parse JSON response.")]
    BadJsonResponse(#[source] reqwest::Error),

    #[error(
        "Status code {} inconsistent with response payload: {}",
        status_code,
        message
    )]
    BadProtocol {
        status_code: StatusCode,
        message: String,
    },

    #[error("Failed to initialise the HTTP client")]
    BuildHttpClient(#[source] reqwest::Error),

    #[error("HTTP request error: {}", message)]
    ReqwestError {
        message: String,
        source: reqwest::Error,
    },

    #[error("An unknown error has occurred: {}", message)]
    Unknown {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}



impl<T> From<OAError<T>> for Error {
    fn from(e: OAError<T>) -> Self {
        match e {
            // Network/HTTP-layer error before we even get a response
            OAError::Reqwest(err) => Error::ReqwestError {
                message: err.to_string(),
                source: err,
            },

            // JSON (de)serialisation error
            OAError::Serde(err) => Error::Unknown {
                message: format!("JSON error: {err}"),
                source: Box::new(err),
            },

            // File/IO error
            OAError::Io(err) => Error::Unknown {
                message: format!("I/O error: {err}"),
                source: Box::new(err),
            },

            OAError::ResponseError(ResponseContent { status, content, .. }) => {
                Error::Api {
                    status_code: status,
                    message: content,
                }
            }
        }
    }
}
