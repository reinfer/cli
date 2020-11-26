use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use crate::{
    errors::{Error, ErrorKind, Result},
    resources::user::Username,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FullName(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelFamily(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransformTag(pub String);

impl FromStr for TransformTag {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        Ok(Self(string.to_owned()))
    }
}

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
        } else if string.split('/').count() == 2 {
            Ok(Identifier::FullName(FullName(string.into())))
        } else {
            Err(ErrorKind::BadBucketIdentifier {
                identifier: string.into(),
            }
            .into())
        }
    }
}

impl Display for FullName {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "{}", self.0)
    }
}

impl Display for Id {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "{}", self.0)
    }
}

impl Display for Identifier {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        match *self {
            Identifier::Id(ref id) => Display::fmt(id, formatter),
            Identifier::FullName(ref full_name) => Display::fmt(full_name, formatter),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NewBucket<'request> {
    pub bucket_type: BucketType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform_tag: Option<&'request TransformTag>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CreateRequest<'request> {
    pub bucket: NewBucket<'request>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CreateResponse {
    pub bucket: Bucket,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GetAvailableResponse {
    pub buckets: Vec<Bucket>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GetResponse {
    pub bucket: Bucket,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum BucketType {
    #[serde(rename = "emails")]
    Emails,
}

impl FromStr for BucketType {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        match string {
            "emails" => Ok(Self::Emails),
            _ => Err(ErrorKind::BadBucketType {
                bucket_type: string.into(),
            }
            .into()),
        }
    }
}

impl Default for BucketType {
    fn default() -> Self {
        Self::Emails
    }
}

impl Display for BucketType {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        match *self {
            Self::Emails => write!(formatter, "emails"),
        }
    }
}
