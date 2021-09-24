use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use crate::{
    error::{Error, Result},
    resources::user::Username,
};

lazy_static! {
    static ref FULL_NAME_REGEX: Regex =
        Regex::new("^[A-Za-z0-9-_]{1,256}/[A-Za-z0-9-_]{1,256}$").unwrap();
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Bucket {
    pub id: Id,
    pub name: Name,
    pub owner: Username,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub transform_tag: Option<TransformTag>,
}

impl Bucket {
    pub fn full_name(&self) -> FullName {
        FullName(format!("{}/{}", self.owner.0, self.name.0))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct FullName(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ModelFamily(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct TransformTag(pub String);

impl FromStr for TransformTag {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        Ok(Self(string.to_owned()))
    }
}

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
        if string.chars().all(|c| c.is_digit(16)) {
            Ok(Identifier::Id(Id(string.into())))
        } else if FULL_NAME_REGEX.is_match(string) {
            Ok(Identifier::FullName(FullName(string.into())))
        } else {
            Err(Error::BadBucketIdentifier {
                identifier: string.into(),
            })
        }
    }
}

impl Display for FullName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.0)
    }
}

impl Display for Id {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.0)
    }
}

impl Display for Identifier {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Identifier::Id(id) => Display::fmt(id, formatter),
            Identifier::FullName(full_name) => Display::fmt(full_name, formatter),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NewBucket<'request> {
    pub bucket_type: BucketType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,
    pub transform_tag: &'request TransformTag,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CreateRequest<'request> {
    pub bucket: NewBucket<'request>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CreateResponse {
    pub bucket: Bucket,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetAvailableResponse {
    pub buckets: Vec<Bucket>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetResponse {
    pub bucket: Bucket,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum BucketType {
    #[serde(rename = "emails")]
    Emails,
}

impl FromStr for BucketType {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        match string {
            "emails" => Ok(Self::Emails),
            _ => Err(Error::BadBucketType {
                bucket_type: string.into(),
            }),
        }
    }
}

impl Default for BucketType {
    fn default() -> Self {
        Self::Emails
    }
}

impl Display for BucketType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match *self {
            Self::Emails => write!(formatter, "emails"),
        }
    }
}

impl FromStr for FullName {
    type Err = Error;
    fn from_str(string: &str) -> Result<Self> {
        if FULL_NAME_REGEX.is_match(string) {
            Ok(FullName(string.into()))
        } else {
            Err(Error::BadBucketName {
                name: string.into(),
            })
        }
    }
}
