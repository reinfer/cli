use reqwest::StatusCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API request failed with {}: {}", status_code, message)]
    Api {
        status_code: StatusCode,
        message: String,
    },

    #[error("Invalid endpoint `{}`", name)]
    BadEndpoint { name: String },

    #[error("Bad token: {}", token)]
    BadToken { token: String },

    #[error("Expected <owner>/<name> or a source id, got: {}", identifier)]
    BadSourceIdentifier { identifier: String },

    #[error("Expected <owner>/<name> or a dataset id, got: {}", identifier)]
    BadDatasetIdentifier { identifier: String },

    #[error("Expected <owner>/<dataset>/<trigger>: {}", identifier)]
    BadTriggerName { identifier: String },

    #[error("Expected a username or user id, got: {}", identifier)]
    BadUserIdentifier { identifier: String },

    #[error("Unknown organisation permission: {}", permission)]
    BadOrganisationPermission { permission: String },

    #[error("Unknown global permission: {}", permission)]
    BadGlobalPermission { permission: String },

    #[error("Expected <owner>/<name> or a bucket id, got: {}", identifier)]
    BadBucketIdentifier { identifier: String },

    #[error("Expected <owner>/<name>, got: {}", name)]
    BadBucketName { name: String },

    #[error("Expected a valid bucket type, got: {}", bucket_type)]
    BadBucketType { bucket_type: String },

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

    #[error("Url parsing error: {}", message)]
    UrlParseError {
        message: String,
        source: url::ParseError,
    },

    #[error("An unknown error has occurred: {}", message)]
    Unknown {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}
