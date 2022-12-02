use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    resources::{
        entity_def::{EntityDef, NewEntityDef},
        label_def::{LabelDef, NewLabelDef},
        label_group::{LabelGroup, NewLabelGroup},
        source::Id as SourceId,
        user::Username,
    },
};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Dataset {
    pub id: Id,
    pub name: Name,
    pub owner: Username,
    pub title: String,
    pub description: String,
    #[serde(rename = "created")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "last_modified")]
    pub updated_at: DateTime<Utc>,
    pub model_family: ModelFamily,
    pub source_ids: Vec<SourceId>,
    pub has_sentiment: bool,
    pub entity_defs: Vec<EntityDef>,
    pub label_defs: Vec<LabelDef>,
    pub label_groups: Vec<LabelGroup>,
}

impl Dataset {
    pub fn full_name(&self) -> FullName {
        FullName(format!("{}/{}", self.owner.0, self.name.0))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct FullName(pub String);

impl FromStr for FullName {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.split('/').count() == 2 {
            Ok(FullName(string.into()))
        } else {
            Err(Error::BadDatasetIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ModelFamily(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ModelVersion(pub u32);

// TODO(mcobzarenco)[3963]: Make `Identifier` into a trait (ensure it still implements
// `FromStr` so we can take T: Identifier as a clap command line argument).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Identifier {
    Id(Id),
    FullName(FullName),
}

impl From<FullName> for Identifier {
    fn from(full_name: FullName) -> Self {
        Identifier::FullName(full_name)
    }
}

impl From<Id> for Identifier {
    fn from(id: Id) -> Self {
        Identifier::Id(id)
    }
}

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Identifier::Id(Id(string.into())))
        } else {
            FullName::from_str(string).map(Identifier::FullName)
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "{}",
            match self {
                Identifier::Id(id) => &id.0,
                Identifier::FullName(full_name) => &full_name.0,
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NewDataset<'request> {
    pub source_ids: &'request [SourceId],

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_sentiment: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_defs: Option<&'request [NewEntityDef]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_defs: Option<&'request [NewLabelDef]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_groups: Option<&'request [NewLabelGroup]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_family: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_annotations_from: Option<&'request str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CreateRequest<'request> {
    pub dataset: NewDataset<'request>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CreateResponse {
    pub dataset: Dataset,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetAvailableResponse {
    pub datasets: Vec<Dataset>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetResponse {
    pub dataset: Dataset,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UpdateDataset<'request> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ids: Option<&'request [SourceId]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct UpdateRequest<'request> {
    pub dataset: UpdateDataset<'request>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct UpdateResponse {
    pub dataset: Dataset,
}
