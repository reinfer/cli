use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::errors::{Error, ErrorKind, Result};

use super::{
    comment::{Comment, CommentFilter, Entity, LabelName, PredictedLabelParts},
    dataset::{FullName as DatasetFullName, Id as DatasetId},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SequenceId(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FullName {
    pub dataset: DatasetFullName,
    pub trigger: Name,
}

impl FromStr for FullName {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        let mut splits = string.split('/');
        match (splits.next(), splits.next(), splits.next(), splits.next()) {
            (Some(owner), Some(dataset), Some(trigger_name), None) => Ok(FullName {
                dataset: DatasetFullName(format!("{}/{}", owner, dataset)),
                trigger: Name(trigger_name.to_owned()),
            }),
            _ => Err(ErrorKind::BadTriggerName {
                identifier: string.into(),
            }
            .into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Trigger {
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
    pub threshold: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserModelVersion(pub u64);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Batch {
    pub results: Vec<TriggerResult>,
    pub filtered: u32,
    pub sequence_id: SequenceId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerResult {
    pub comment: Comment,
    pub sequence_id: SequenceId,
    pub labels: Option<Vec<PredictedLabelParts>>,
    pub entities: Option<Vec<Entity>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GetResponse {
    pub triggers: Vec<Trigger>,
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
