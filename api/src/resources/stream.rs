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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LabelFilter {
    pub label: LabelName,
    pub model_version: UserModelVersion,
    pub threshold: NotNan<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserModelVersion(pub u64);

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
