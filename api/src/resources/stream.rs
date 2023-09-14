use chrono::{DateTime, Utc};
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::error::{Error, Result};

use super::{
    comment::{Comment, CommentFilter, Entity, PredictedLabel, Uid as CommentUid},
    dataset::{FullName as DatasetFullName, Id as DatasetId},
    label_def::Name as LabelName,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct SequenceId(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct FullName {
    pub dataset: DatasetFullName,
    pub stream: Name,
}

impl FromStr for FullName {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        let mut splits = string.split('/');
        match (splits.next(), splits.next(), splits.next(), splits.next()) {
            (Some(owner), Some(dataset), Some(stream_name), None) => Ok(FullName {
                dataset: DatasetFullName(format!("{owner}/{dataset}")),
                stream: Name(stream_name.to_owned()),
            }),
            _ => Err(Error::BadStreamName {
                identifier: string.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PutStreamRequest<'request> {
    pub stream: &'request NewStream,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutStreamResponse {
    pub stream: Stream,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewStream {
    pub name: Name,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_filter: Option<CommentFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<StreamModel>,
}

impl NewStream {
    pub fn set_model_version(&mut self, model_version: &UserModelVersion) {
        if let Some(model) = &mut self.model {
            model.version = model_version.clone()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamModel {
    pub version: UserModelVersion,
    pub label_thresholds: Vec<StreamLabelThreshold>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamLabelThreshold {
    name: Vec<String>,
    threshold: NotNan<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Stream {
    pub id: Id,
    pub dataset_id: DatasetId,
    pub name: Name,
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[serde(default)]
    pub comment_filter: CommentFilter,

    #[serde(rename = "label_threshold_filter")]
    pub label_filter: Option<LabelFilter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<StreamModel>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LabelFilter {
    pub label: LabelName,
    pub model_version: UserModelVersion,
    pub threshold: NotNan<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserModelVersion(pub u64);

impl FromStr for UserModelVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.parse::<u64>() {
            Ok(version) => Ok(UserModelVersion(version)),
            Err(_) => Err(Error::BadStreamModelVersion {
                version: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Batch {
    pub results: Vec<StreamResult>,
    pub filtered: u32,
    pub sequence_id: SequenceId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamResult {
    pub comment: Comment,
    pub sequence_id: SequenceId,
    pub labels: Option<Vec<PredictedLabel>>,
    pub entities: Option<Vec<Entity>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GetResponse {
    pub streams: Vec<Stream>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FetchRequest {
    pub size: u32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdvanceRequest {
    pub sequence_id: SequenceId,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ResetRequest {
    pub to_comment_created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TagExceptionsRequest<'request> {
    pub exceptions: &'request [StreamException<'request>],
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamException<'request> {
    pub metadata: StreamExceptionMetadata<'request>,
    pub uid: &'request CommentUid,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamExceptionMetadata<'request> {
    pub r#type: &'request String,
}
