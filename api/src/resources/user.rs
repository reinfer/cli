use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
};

use super::project::ProjectName;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

impl FromStr for Id {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.chars().all(|c| c.is_ascii_hexdigit()) {
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
pub enum Identifier {
    Id(Id),
}

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Identifier::Id(Id(string.into())))
        } else {
            Err(Error::BadUserIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct User {
    pub id: Id,
    pub username: Username,
    pub email: Email,
    #[serde(rename = "created")]
    pub created_at: DateTime<Utc>,
    pub global_permissions: HashSet<GlobalPermission>,
    #[serde(rename = "organisation_permissions")]
    pub project_permissions: HashMap<ProjectName, HashSet<ProjectPermission>>,
    pub sso_global_permissions: HashSet<GlobalPermission>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NewUser<'r> {
    pub username: &'r Username,
    pub email: &'r Email,
    pub global_permissions: &'r [GlobalPermission],
    #[serde(rename = "organisation_permissions")]
    pub project_permissions: &'r HashMap<ProjectName, HashSet<ProjectPermission>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModifiedPermissions<'r> {
    #[serde(
        rename = "organisation_permissions",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub project_permissions: &'r HashMap<ProjectName, HashSet<ProjectPermission>>,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetResponse {
    pub user: User,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct WelcomeEmailResponse {}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ProjectPermission {
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

    #[serde(rename = "streams-admin")]
    StreamsAdmin,

    #[serde(rename = "streams-consume")]
    StreamsConsume,

    #[serde(rename = "streams-read")]
    StreamsRead,

    #[serde(rename = "streams-write")]
    StreamsWrite,

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

impl FromStr for ProjectPermission {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        serde_json::de::from_str(&format!("\"{string}\"")).map_err(|_| {
            Error::BadProjectPermission {
                permission: string.into(),
            }
        })
    }
}

#[derive(Debug, Clone, DeserializeFromStr, SerializeDisplay, PartialEq, Eq, Hash)]
pub enum GlobalPermission {
    Root,
    Debug,
    Demo,
    SubscriptionsRead,
    ArtefactsRead,
    LegacyDialog,
    SupportTenantAdmin,
    SupportUsersWrite,
    DeploymentQuotaWrite,
    TenantAdmin,
    TenantQuotaWrite,
    Unknown(Box<str>),
}

const ROOT_AS_STR: &str = "root";
const DEBUG_AS_STR: &str = "debug";
const DEMO_AS_STR: &str = "demo";
const SUBSCRIPTIONS_READ_AS_STR: &str = "subscriptions-read";
const ARTEFACTS_READ_AS_STR: &str = "artefacts-read";
const LEGACY_DIALOG_AS_STR: &str = "dialog";
const SUPPORT_TENANT_ADMIN_AS_STR: &str = "support-tenant-admin";
const SUPPORT_USERS_WRITE_AS_STR: &str = "support-users-write";
const DEPLOYMENT_QUOTA_WRITE_AS_STR: &str = "deployment-quota-write";
const TENANT_ADMIN_AS_STR: &str = "tenant-admin";
const TENANT_QUOTA_WRITE_AS_STR: &str = "tenant-quota-write";

impl FromStr for GlobalPermission {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        Ok(match string {
            ROOT_AS_STR => GlobalPermission::Root,
            DEBUG_AS_STR => GlobalPermission::Debug,
            DEMO_AS_STR => GlobalPermission::Demo,
            SUBSCRIPTIONS_READ_AS_STR => GlobalPermission::SubscriptionsRead,
            ARTEFACTS_READ_AS_STR => GlobalPermission::ArtefactsRead,
            LEGACY_DIALOG_AS_STR => GlobalPermission::LegacyDialog,
            SUPPORT_TENANT_ADMIN_AS_STR => GlobalPermission::SupportTenantAdmin,
            SUPPORT_USERS_WRITE_AS_STR => GlobalPermission::SupportUsersWrite,
            TENANT_ADMIN_AS_STR => GlobalPermission::TenantAdmin,
            TENANT_QUOTA_WRITE_AS_STR => GlobalPermission::TenantQuotaWrite,
            DEPLOYMENT_QUOTA_WRITE_AS_STR => GlobalPermission::DeploymentQuotaWrite,
            value => GlobalPermission::Unknown(value.into()),
        })
    }
}

impl Display for GlobalPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GlobalPermission::Root => ROOT_AS_STR,
                GlobalPermission::Debug => DEBUG_AS_STR,
                GlobalPermission::Demo => DEMO_AS_STR,
                GlobalPermission::SubscriptionsRead => SUBSCRIPTIONS_READ_AS_STR,
                GlobalPermission::ArtefactsRead => ARTEFACTS_READ_AS_STR,
                GlobalPermission::LegacyDialog => LEGACY_DIALOG_AS_STR,
                GlobalPermission::SupportTenantAdmin => SUPPORT_TENANT_ADMIN_AS_STR,
                GlobalPermission::SupportUsersWrite => SUPPORT_USERS_WRITE_AS_STR,
                GlobalPermission::TenantAdmin => TENANT_ADMIN_AS_STR,
                GlobalPermission::TenantQuotaWrite => TENANT_QUOTA_WRITE_AS_STR,
                GlobalPermission::DeploymentQuotaWrite => DEPLOYMENT_QUOTA_WRITE_AS_STR,
                GlobalPermission::Unknown(value) => value.as_ref(),
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize)]
pub struct UpdateUser {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organisation_permissions: Option<HashMap<ProjectName, Vec<ProjectPermission>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_permissions: Option<Vec<GlobalPermission>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct PostUserRequest<'request> {
    pub user: &'request UpdateUser,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PostUserResponse {
    pub user: User,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_global_permissions_json_round_trip() {
        let global_permissions = vec![
            GlobalPermission::Root,
            GlobalPermission::Debug,
            GlobalPermission::Demo,
            GlobalPermission::SubscriptionsRead,
            GlobalPermission::ArtefactsRead,
            GlobalPermission::LegacyDialog,
            GlobalPermission::SupportTenantAdmin,
            GlobalPermission::SupportUsersWrite,
            GlobalPermission::TenantAdmin,
            GlobalPermission::TenantQuotaWrite,
            GlobalPermission::DeploymentQuotaWrite,
            GlobalPermission::Unknown("new-perm".to_string().into_boxed_str()),
        ];
        let global_permissions_as_json_str = serde_json::to_string(&global_permissions).unwrap();

        assert_eq!("[\"root\",\"debug\",\"demo\",\"subscriptions-read\",\"artefacts-read\",\"dialog\",\"support-tenant-admin\",\"support-users-write\",\"tenant-admin\",\"tenant-quota-write\",\"deployment-quota-write\",\"new-perm\"]", global_permissions_as_json_str);

        let global_permissions_from_json_str: Vec<GlobalPermission> =
            serde_json::from_str(&global_permissions_as_json_str).unwrap();

        assert_eq!(global_permissions, global_permissions_from_json_str);
    }
    #[test]
    fn unknown_project_permission_roundtrips() {
        let unknown_permission = ProjectPermission::from_str("unknown").unwrap();
        match &unknown_permission {
            ProjectPermission::Unknown(error) => assert_eq!(&**error, "unknown"),
            _ => panic!("Expected error to be parsed as Unknown(..)"),
        }

        assert_eq!(
            &serde_json::ser::to_string(&unknown_permission).unwrap(),
            "\"unknown\""
        )
    }

    #[test]
    #[should_panic]
    fn specific_project_permission_roundtrips() {
        // TODO(jcalero)[RE-978] This test was written to showcase bug RE-978. It demonstrates that
        // deserialization of an `ProjectPermission` does not parse correctly.
        let permission = ProjectPermission::DatasetsRead;

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
