use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

impl FromStr for Id {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.chars().all(|c| c.is_digit(16)) {
            Ok(Id(string.into()))
        } else {
            Err(Error::BadUserIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Username(pub String);

impl FromStr for Username {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            Ok(Username(string.into()))
        } else {
            Err(Error::BadUserIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Email(pub String);

impl FromStr for Email {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        Ok(Email(string.into()))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Organisation(pub String);

impl FromStr for Organisation {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            Ok(Organisation(string.into()))
        } else {
            Err(Error::BadUserIdentifier {
                identifier: string.into(),
            })
        }
    }
}

// TODO(mcobzarenco)[3963]: Make `Identifier` into a trait (ensure it still implements
// `FromStr` so we can take T: Identifier as a clap command line argument).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Identifier {
    Id(Id),
    Username(Username),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct User {
    pub id: Id,
    pub username: Username,
    pub email: Email,
    #[serde(rename = "created")]
    pub created_at: DateTime<Utc>,
    pub global_permissions: HashSet<GlobalPermission>,
    pub organisation_permissions: HashMap<Organisation, HashSet<OrganisationPermission>>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NewUser<'r> {
    pub username: &'r Username,
    pub email: &'r Email,
    pub global_permissions: &'r [GlobalPermission],
    pub organisation_permissions: &'r HashMap<Organisation, HashSet<OrganisationPermission>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModifiedPermissions<'r> {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub organisation_permissions: &'r HashMap<Organisation, HashSet<OrganisationPermission>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub global_permissions: Vec<&'r GlobalPermission>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CreateRequest<'request> {
    pub user: NewUser<'request>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CreateResponse {
    pub user: User,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetAvailableResponse {
    pub users: Vec<User>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetCurrentResponse {
    pub user: User,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum OrganisationPermission {
    // TODO(jcalero)[RE-978] There is a bug with the implementation of this enum that causes
    // deserialization of non-Unknown properties to fail. See
    // [RE-978](https://reinfer.atlassian.net/browse/RE-978) for more info.
    #[serde(rename = "sources-add-comments")]
    CommentsAdmin,

    #[serde(rename = "datasets-admin")]
    DatasetsAdmin,

    #[serde(rename = "voc")]
    DatasetsWrite,

    #[serde(rename = "datasets-review")]
    DatasetsReview,

    #[serde(rename = "voc-readonly")]
    DatasetsRead,

    #[serde(rename = "datasets-export")]
    DatasetsExport,

    #[serde(rename = "sources-admin")]
    SourcesAdmin,

    #[serde(rename = "sources-translate")]
    SourcesTranslate,

    #[serde(rename = "sources-read")]
    SourcesRead,

    #[serde(rename = "sources-read-sensitive")]
    SourcesReadSensitive,

    #[serde(rename = "triggers-admin")]
    TriggersAdmin,

    #[serde(rename = "triggers-consume")]
    TriggersConsume,

    #[serde(rename = "triggers-read")]
    TriggersRead,

    #[serde(rename = "triggers-write")]
    TriggersWrite,

    #[serde(rename = "users-read")]
    UsersRead,

    #[serde(rename = "users-write")]
    UsersWrite,

    #[serde(rename = "buckets-read")]
    BucketsRead,

    #[serde(rename = "buckets-write")]
    BucketsWrite,

    #[serde(rename = "buckets-append")]
    BucketsAppend,

    #[serde(rename = "files-write")]
    FilesWrite,

    #[serde(rename = "appliance-config-read")]
    ApplianceConfigRead,

    #[serde(rename = "appliance-config-write")]
    ApplianceConfigWrite,

    #[serde(rename = "integrations-read")]
    IntegrationsRead,

    #[serde(rename = "integrations-write")]
    IntegrationsWrite,

    Unknown(Box<str>),
}

impl FromStr for OrganisationPermission {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        serde_json::de::from_str(&format!("\"{}\"", string)).map_err(|_| {
            Error::BadOrganisationPermission {
                permission: string.into(),
            }
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum GlobalPermission {
    // TODO(jcalero)[RE-978] There is a bug with the implementation of this enum that causes
    // deserialization of non-Unknown properties to fail. See
    // [RE-978](https://reinfer.atlassian.net/browse/RE-978) for more info.
    #[serde(rename = "root")]
    Root,

    #[serde(rename = "debug")]
    Debug,

    #[serde(rename = "demo")]
    Demo,

    #[serde(rename = "subscriptions-read")]
    SubscriptionsRead,

    #[serde(rename = "artefacts-read")]
    ArtefactsRead,

    #[serde(rename = "dialog")]
    LegacyDialog,

    Unknown(Box<str>),
}

impl FromStr for GlobalPermission {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        serde_json::de::from_str(&format!("\"{}\"", string)).map_err(|_| {
            Error::BadGlobalPermission {
                permission: string.into(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_organisation_permission_roundtrips() {
        let unknown_permission = OrganisationPermission::from_str("unknown").unwrap();
        match &unknown_permission {
            OrganisationPermission::Unknown(error) => assert_eq!(&**error, "unknown"),
            _ => panic!("Expected error to be parsed as Unknown(..)"),
        }

        assert_eq!(
            &serde_json::ser::to_string(&unknown_permission).unwrap(),
            "\"unknown\""
        )
    }

    #[test]
    #[should_panic]
    fn specific_organisation_permission_roundtrips() {
        // TODO(jcalero)[RE-978] This test was written to showcase bug RE-978. It demonstrates that
        // deserialization of an `OrganisationPermission` does not parse correctly.
        let permission = OrganisationPermission::DatasetsRead;

        assert_eq!(
            &serde_json::ser::to_string(&permission).unwrap(),
            "\"voc-readonly\""
        )
    }

    #[test]
    fn unknown_global_permission_roundtrips() {
        let unknown_permission = GlobalPermission::from_str("unknown").unwrap();
        match &unknown_permission {
            GlobalPermission::Unknown(error) => assert_eq!(&**error, "unknown"),
            _ => panic!("Expected error to be parsed as Unknown(..)"),
        }

        assert_eq!(
            &serde_json::ser::to_string(&unknown_permission).unwrap(),
            "\"unknown\""
        )
    }
}
