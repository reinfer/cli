use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
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

    #[serde(rename = "_kind")]
    pub kind: SourceKind,
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

#[derive(Debug, Clone, SerializeDisplay, DeserializeFromStr, PartialEq, Eq, Hash)]
pub enum SourceKind {
    Call,
    Chat,
    Unknown(Box<str>),
}

impl FromStr for SourceKind {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        Ok(match string {
            "call" => SourceKind::Call,
            "chat" => SourceKind::Chat,
            value => SourceKind::Unknown(value.into()),
        })
    }
}

impl Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SourceKind::Call => "call",
                SourceKind::Chat => "chat",
                SourceKind::Unknown(value) => value.as_ref(),
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

    #[serde(skip_serializing_if = "Option::is_none", rename = "_kind")]
    pub kind: Option<&'request SourceKind>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_kind_roundtrips() {
        assert_eq!(SourceKind::Call, SourceKind::from_str("call").unwrap());
        assert_eq!(
            &serde_json::ser::to_string(&SourceKind::Call).unwrap(),
            "\"call\""
        );

        assert_eq!(SourceKind::Chat, SourceKind::from_str("chat").unwrap());
        assert_eq!(
            &serde_json::ser::to_string(&SourceKind::Chat).unwrap(),
            "\"chat\""
        );
    }

    #[test]
    fn unknown_source_kind_roundtrips() {
        let kind = SourceKind::from_str("unknown").unwrap();
        match &kind {
            SourceKind::Unknown(error) => assert_eq!(&**error, "unknown"),
            _ => panic!("Expected error to be parsed as Unknown(..)"),
        }

        assert_eq!(&serde_json::ser::to_string(&kind).unwrap(), "\"unknown\"")
    }
}
