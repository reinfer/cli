use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{Email, ProjectName};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewIntegration {
    pub title: Title,
    pub enabled: bool,
    pub configuration: Configuration,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Integration {
    pub id: Id,
    pub owner: ProjectName,
    pub name: FullName,
    pub title: Title,
    #[serde(rename = "type")]
    pub integration_type: IntegrationType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub enabled: bool,
    pub disabled_reason: Option<DisabledReason>,
    pub configuration: Configuration,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct FullName(pub String);

impl FromStr for FullName {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.split('/').count() == 2 {
            Ok(FullName(string.into()))
        } else {
            Err(Error::BadIntegrationIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Title(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationType {
    ExchangeOnline,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisabledReason {
    User,
    Quota,
    SyncError,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Configuration {
    pub connection: Option<Connection>,
    pub mailboxes: Vec<Mailbox>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Connection {
    access: AccessType,
    application: ApplicationType,
    ews_endpoint: Option<String>,
    build_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AccessType {
    UserAccess(UserAccessConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct UserAccessConfig {
    username: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApplicationType {
    Custom(CustomApplicationConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct CustomApplicationConfig {
    client_id: String,
    tenant_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Mailbox {
    pub disabled_reason: Option<DisabledReason>,
    pub email: Email,
    pub enabled: bool,
    pub folder_allowlist: Option<Vec<Vec<String>>>,
    pub folder_denylist: Option<Vec<Vec<String>>>,
    pub participant_allowlist: Option<Vec<Email>>,
    pub participant_denylist: Option<Vec<Email>>,
    pub participant_domain_allowlist: Option<Vec<String>>,
    pub participant_domain_denylist: Option<Vec<String>>,
    pub start_timestamp: Option<DateTime<Utc>>,
    pub bucket_specification: BucketSpecification,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BucketSpecification {
    project_name: ProjectName,
    name: String,
    title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GetIntegrationsResponse {
    pub integrations: Vec<Integration>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GetIntegrationResponse {
    pub integration: Integration,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PostIntegrationResponse {
    pub integration: Integration,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PutIntegrationResponse {
    pub integration: Integration,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PostIntegrationRequest {
    pub integration: NewIntegration,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PutIntegrationRequest {
    pub integration: NewIntegration,
}
