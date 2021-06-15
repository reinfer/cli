use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{
    error::{Error, Result},
    resources::{comment::EntityName, source::Id as SourceId, user::Username},
};
use serde_json::Error as JsonError;

#[derive(Debug, Clone, Deserialize, Serialize)]
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
}

impl Dataset {
    pub fn full_name(&self) -> FullName {
        FullName(format!("{}/{}", self.owner.0, self.name.0))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelFamily(pub String);

// TODO(mcobzarenco)[3963]: Make `Identifier` into a trait (ensure it still implements
// `FromStr` so we can take T: Identifier as a clap command line argument).
#[derive(Debug, Clone, Deserialize, Serialize)]
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
        if string.chars().all(|c| c.is_digit(16)) {
            Ok(Identifier::Id(Id(string.into())))
        } else {
            FullName::from_str(string).map(Identifier::FullName)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityDef {
    name: EntityName,
    title: String,
    inherits_from: Vec<String>,
    trainable: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EntityDefs(Vec<EntityDef>);

impl EntityDefs {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromStr for EntityDefs {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        serde_json::from_str(string).map_err(|error: JsonError| Error::BadEntityDef {
            entity_def: string.to_owned(),
            source: error,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct NewDataset<'request> {
    pub source_ids: &'request [SourceId],

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_sentiment: Option<bool>,

    #[serde(skip_serializing_if = "EntityDefs::is_empty")]
    pub entity_defs: &'request EntityDefs,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_family: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_annotations_from: Option<&'request str>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CreateRequest<'request> {
    pub dataset: NewDataset<'request>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateResponse {
    pub dataset: Dataset,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GetAvailableResponse {
    pub datasets: Vec<Dataset>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GetResponse {
    pub dataset: Dataset,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateDataset<'request> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ids: Option<&'request [SourceId]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UpdateRequest<'request> {
    pub dataset: UpdateDataset<'request>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UpdateResponse {
    pub dataset: Dataset,
}
