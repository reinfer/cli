use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use crate::{
    error::{Error, Result},
    resources::bucket::Id as BucketId,
    resources::user::Username,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Source {
    pub id: Id,
    pub owner: Username,
    pub name: Name,
    pub title: String,
    pub description: String,
    pub language: String,
    pub should_translate: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Source {
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
            Err(Error::BadSourceIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

// TODO(mcobzarenco)[3963]: Make `Identifier` into a trait (ensure it still implements
// `FromStr` so we can take T: Identifier as a clap command line argument).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Identifier {
    Id(Id),
    FullName(FullName),
}

impl From<Id> for Identifier {
    fn from(id: Id) -> Self {
        Identifier::Id(id)
    }
}

impl From<FullName> for Identifier {
    fn from(full_name: FullName) -> Self {
        Identifier::FullName(full_name)
    }
}

impl<'a> From<&'a Source> for Identifier {
    fn from(source: &Source) -> Self {
        Identifier::FullName(source.full_name())
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

impl Display for Identifier {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(
            formatter,
            "{}",
            match *self {
                Identifier::Id(ref id) => &id.0,
                Identifier::FullName(ref full_name) => &full_name.0,
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct NewSource<'request> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub should_translate: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_id: Option<BucketId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive_properties: Option<Vec<&'request str>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub(crate) struct CreateRequest<'request> {
    pub source: NewSource<'request>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CreateResponse {
    pub source: Source,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetAvailableResponse {
    pub sources: Vec<Source>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetResponse {
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct UpdateSource<'request> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub should_translate: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_id: Option<BucketId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive_properties: Option<Vec<&'request str>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub(crate) struct UpdateRequest<'request> {
    pub source: UpdateSource<'request>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct UpdateResponse {
    pub source: Source,
}
