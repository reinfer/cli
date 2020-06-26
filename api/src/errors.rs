use failchain::{BoxedError, ChainErrorKind};
use failure::Fail;
use reqwest::StatusCode;
use std::result::Result as StdResult;

pub type Error = BoxedError<ErrorKind>;
pub type Result<T> = StdResult<T, Error>;

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "API request failed with {}: {}", status_code, message)]
    Api {
        status_code: StatusCode,
        message: String,
    },

    #[fail(display = "Invalid endpoint `{}`", name)]
    BadEndpoint { name: String },

    #[fail(display = "Bad token: {}", token)]
    BadToken { token: String },

    #[fail(
        display = "Expected <owner>/<name> or a source id, got: {}",
        identifier
    )]
    BadSourceIdentifier { identifier: String },

    #[fail(
        display = "Expected <owner>/<name> or a dataset id, got: {}",
        identifier
    )]
    BadDatasetIdentifier { identifier: String },

    #[fail(display = "Expected <owner>/<dataset>/<trigger>: {}", identifier)]
    BadTriggerName { identifier: String },

    #[fail(display = "Expected a username or user id, got: {}", identifier)]
    BadUserIdentifier { identifier: String },

    #[fail(display = "Unknown organisation permission: {}", permission)]
    BadOrganisationPermission { permission: String },

    #[fail(display = "Unknown global permission: {}", permission)]
    BadGlobalPermission { permission: String },

    #[fail(
        display = "Expected <owner>/<name> or a bucket id, got: {}",
        identifier
    )]
    BadBucketIdentifier { identifier: String },

    #[fail(display = "Expected a valid bucket type, got: {}", bucket_type)]
    BadBucketType { bucket_type: String },

    #[fail(display = "Could not parse JSON response.")]
    BadJsonResponse,

    #[fail(
        display = "Status code {} inconsistent with response payload: {}",
        status_code, message
    )]
    BadProtocol {
        status_code: StatusCode,
        message: String,
    },

    #[fail(display = "Failed to initialise the HTTP client")]
    BuildHttpClient,

    #[fail(display = "An unknown error has occurred: {}", message)]
    Unknown { message: String },
}

impl ChainErrorKind for ErrorKind {
    type Error = Error;
}
