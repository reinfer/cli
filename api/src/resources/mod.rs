pub mod audit;
pub mod bucket;
pub mod bucket_statistics;
pub mod comment;
pub mod dataset;
pub mod documents;
pub mod email;
pub mod entity_def;
pub mod integration;
pub mod label_def;
pub mod label_group;
pub mod project;
pub mod quota;
pub mod source;
pub mod statistics;
pub mod stream;
pub mod tenant_id;
pub mod user;
pub mod validation;

use crate::error::{Error, Result};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "status")]
pub(crate) enum Response<SuccessT> {
    #[serde(rename = "ok")]
    Success(SuccessT),

    #[serde(rename = "error")]
    Error(ApiError),
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ApiError {
    message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EmptySuccess {}

impl ApiError {
    fn into_error_kind(self, status_code: StatusCode) -> Error {
        Error::Api {
            status_code,
            message: self.message.unwrap_or_default(),
        }
    }
}

impl<'de, SuccessT: Deserialize<'de>> Response<SuccessT> {
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
                        message: error.message.unwrap_or_default(),
                    })
                } else {
                    Err(error.into_error_kind(status_code))
                }
            }
        }
    }
}
