pub mod bucket;
pub mod comment;
pub mod dataset;
pub mod email;
pub mod source;
pub mod statistics;
pub mod trigger;
pub mod user;

use crate::error::{Error, Result};
use reqwest::StatusCode;
use serde::Deserialize;

#[serde(tag = "status")]
#[derive(Debug, Clone, Deserialize)]
pub(crate) enum Response<SuccessT, ErrorT: ApiError> {
    #[serde(rename = "ok")]
    Success(SuccessT),

    #[serde(rename = "error")]
    Error(ErrorT),
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SimpleApiError {
    message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EmptySuccess {}

pub(crate) trait ApiError {
    fn into_error_kind(self, status_code: StatusCode) -> Error;
    fn message(&self) -> Option<&str>;
}

impl ApiError for SimpleApiError {
    fn message(&self) -> Option<&str> {
        match &self.message {
            Some(message) => Some(message.as_str()),
            None => None,
        }
    }

    fn into_error_kind(self, status_code: StatusCode) -> Error {
        Error::Api {
            status_code,
            message: self.message.unwrap_or_else(String::new),
        }
    }
}

impl<'de, SuccessT: Deserialize<'de>, ErrorT: ApiError + Deserialize<'de>>
    Response<SuccessT, ErrorT>
{
    pub fn into_result(self, status_code: StatusCode) -> Result<SuccessT> {
        match self {
            Response::Success(success) => {
                if status_code.is_success() {
                    Ok(success)
                } else {
                    Err(Error::BadProtocol {
                        status_code,
                        message: String::new(),
                    })
                }
            }
            Response::Error(error) => {
                if status_code.is_success() {
                    Err(Error::BadProtocol {
                        status_code,
                        message: error.message().unwrap_or("").to_owned(),
                    })
                } else {
                    Err(error.into_error_kind(status_code))
                }
            }
        }
    }
}
