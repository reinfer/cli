use chrono::{DateTime, Utc};

use crate::{Continuation, DatasetId, DatasetName, Email, ProjectName, UserId, Username};

use super::{comment::CommentTimestampFilter, project::Id as ProjectId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditQueryFilter {
    pub timestamp: CommentTimestampFilter,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct AuditEventId(pub String);

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct AuditEventType(pub String);

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct AuditTenantName(pub String);

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct AuditTenantId(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditQueryRequest {
    pub filter: AuditQueryFilter,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub continuation: Option<Continuation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditEvent {
    actor_user_id: UserId,
    actor_tenant_id: AuditTenantId,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    dataset_ids: Vec<DatasetId>,
    event_id: AuditEventId,
    event_type: AuditEventType,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    project_ids: Vec<ProjectId>,
    tenant_ids: Vec<AuditTenantId>,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrintableAuditEvent {
    pub actor_email: Email,
    pub actor_tenant_name: AuditTenantName,
    pub event_type: AuditEventType,
    pub dataset_names: Vec<DatasetName>,
    pub event_id: AuditEventId,
    pub project_names: Vec<ProjectName>,
    pub tenant_names: Vec<AuditTenantName>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuditDataset {
    id: DatasetId,
    name: DatasetName,
    project_id: ProjectId,
    title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuditProject {
    id: ProjectId,
    name: ProjectName,
    tenant_id: AuditTenantId,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuditTenant {
    id: AuditTenantId,
    name: AuditTenantName,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuditUser {
    display_name: Username,
    email: Email,
    id: UserId,
    tenant_id: AuditTenantId,
    username: Username,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditQueryResponse {
    audit_events: Vec<AuditEvent>,
    projects: Vec<AuditProject>,
    pub continuation: Option<Continuation>,
    datasets: Vec<AuditDataset>,
    tenants: Vec<AuditTenant>,
    users: Vec<AuditUser>,
}

impl AuditQueryResponse {
    pub fn into_iter_printable(self) -> QueryResponseIterator {
        QueryResponseIterator {
            response: self,
            index: 0,
        }
    }

    fn get_user(&self, user_id: &UserId) -> Option<&AuditUser> {
        self.users.iter().find(|user| user.id == *user_id)
    }

    fn get_dataset(&self, dataset_id: &DatasetId) -> Option<&AuditDataset> {
        self.datasets
            .iter()
            .find(|dataset| dataset.id == *dataset_id)
    }

    fn get_project(&self, project_id: &ProjectId) -> Option<&AuditProject> {
        self.projects
            .iter()
            .find(|project| project.id == *project_id)
    }

    fn get_tenant(&self, tenant_id: &AuditTenantId) -> Option<&AuditTenant> {
        self.tenants.iter().find(|tenant| tenant.id == *tenant_id)
    }
}

pub struct QueryResponseIterator {
    response: AuditQueryResponse,
    index: usize,
}

impl Iterator for QueryResponseIterator {
    type Item = PrintableAuditEvent;
    fn next(&mut self) -> Option<Self::Item> {
        let event = self.response.audit_events.get(self.index)?;

        let actor_email = &self
            .response
            .get_user(&event.actor_user_id)
            .unwrap_or_else(|| panic!("Could not find user for id `{}`", event.actor_user_id.0))
            .email;

        let dataset_names = event
            .dataset_ids
            .iter()
            .map(|dataset_id| {
                &self
                    .response
                    .get_dataset(dataset_id)
                    .unwrap_or_else(|| panic!("Could not get dataset for id `{}`", dataset_id.0))
                    .name
            })
            .cloned()
            .collect();

        let project_names = event
            .project_ids
            .iter()
            .map(|project_id| {
                &self
                    .response
                    .get_project(project_id)
                    .unwrap_or_else(|| panic!("Could not get project for id `{}`", project_id.0))
                    .name
            })
            .cloned()
            .collect();

        let tenant_names = event
            .tenant_ids
            .iter()
            .map(|tenant_id| {
                &self
                    .response
                    .get_tenant(tenant_id)
                    .unwrap_or_else(|| panic!("Could not get tenant for id `{}`", tenant_id.0))
                    .name
            })
            .cloned()
            .collect();

        let actor_tenant_name = &self
            .response
            .get_tenant(&event.actor_tenant_id)
            .unwrap_or_else(|| panic!("Could not get tenant for id `{}`", event.actor_tenant_id.0))
            .name;

        self.index += 1;

        Some(PrintableAuditEvent {
            event_type: event.event_type.clone(),
            actor_tenant_name: actor_tenant_name.clone(),
            event_id: event.event_id.clone(),
            timestamp: event.timestamp,
            actor_email: actor_email.clone(),
            dataset_names,
            project_names,
            tenant_names,
        })
    }
}
