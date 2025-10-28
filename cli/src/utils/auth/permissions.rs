//! Permission types with FromStr implementations for CLI usage
//!
//! This module provides permission wrappers that implement FromStr,
//! making them work seamlessly with StructOpt for command-line argument parsing.

use anyhow::Result;
use std::str::FromStr;

/// Global permission wrapper that implements FromStr for CLI usage
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalPermission(pub openapi::models::GlobalPermission);

impl FromStr for GlobalPermission {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let perm = match s {
            "root" => openapi::models::GlobalPermission::Root,
            "debug" => openapi::models::GlobalPermission::Debug,
            "demo" => openapi::models::GlobalPermission::Demo,
            "audit-read" => openapi::models::GlobalPermission::AuditRead,
            "artefacts-read" => openapi::models::GlobalPermission::ArtefactsRead,
            "support-users-write" => openapi::models::GlobalPermission::SupportUsersWrite,
            "tenant-users-read" => openapi::models::GlobalPermission::TenantUsersRead,
            "tenant-admin" => openapi::models::GlobalPermission::TenantAdmin,
            "support-tenant-admin" => openapi::models::GlobalPermission::SupportTenantAdmin,
            "feature-qos" => openapi::models::GlobalPermission::FeatureQos,
            "feature-pretrained-labels" => {
                openapi::models::GlobalPermission::FeaturePretrainedLabels
            }
            "tenant-quota-read" => openapi::models::GlobalPermission::TenantQuotaRead,
            "tenant-quota-write" => openapi::models::GlobalPermission::TenantQuotaWrite,
            "deployment-quota-read" => openapi::models::GlobalPermission::DeploymentQuotaRead,
            "deployment-quota-write" => openapi::models::GlobalPermission::DeploymentQuotaWrite,
            "role-cm-tenant-admin" => openapi::models::GlobalPermission::RoleCmTenantAdmin,
            "dialog" => openapi::models::GlobalPermission::Dialog,
            "projects-admin" => openapi::models::GlobalPermission::ProjectsAdmin,
            _ => return Err(anyhow::anyhow!("Invalid global permission: {}", s)),
        };
        Ok(GlobalPermission(perm))
    }
}

impl From<GlobalPermission> for openapi::models::GlobalPermission {
    fn from(perm: GlobalPermission) -> Self {
        perm.0
    }
}

/// Project permission wrapper that implements FromStr for CLI usage
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectPermission(pub openapi::models::ProjectPermission);

impl FromStr for ProjectPermission {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let perm = match s {
            "sources-add-comments" => openapi::models::ProjectPermission::SourcesAddComments,
            "datasets-admin" => openapi::models::ProjectPermission::DatasetsAdmin,
            "voc" => openapi::models::ProjectPermission::Voc,
            "datasets-review" => openapi::models::ProjectPermission::DatasetsReview,
            "voc-readonly" => openapi::models::ProjectPermission::VocReadonly,
            "datasets-model-admin" => openapi::models::ProjectPermission::DatasetsModelAdmin,
            "datasets-export" => openapi::models::ProjectPermission::DatasetsExport,
            "dashboards-write" => openapi::models::ProjectPermission::DashboardsWrite,
            "sources-admin" => openapi::models::ProjectPermission::SourcesAdmin,
            "sources-translate" => openapi::models::ProjectPermission::SourcesTranslate,
            "sources-read" => openapi::models::ProjectPermission::SourcesRead,
            "sources-read-sensitive" => openapi::models::ProjectPermission::SourcesReadSensitive,
            "triggers-admin" => openapi::models::ProjectPermission::TriggersAdmin,
            "triggers-consume" => openapi::models::ProjectPermission::TriggersConsume,
            "triggers-read" => openapi::models::ProjectPermission::TriggersRead,
            "triggers-write" => openapi::models::ProjectPermission::TriggersWrite,
            "users-read" => openapi::models::ProjectPermission::UsersRead,
            "users-write" => openapi::models::ProjectPermission::UsersWrite,
            "buckets-read" => openapi::models::ProjectPermission::BucketsRead,
            "buckets-write" => openapi::models::ProjectPermission::BucketsWrite,
            "buckets-items-read" => openapi::models::ProjectPermission::BucketsItemsRead,
            "buckets-append" => openapi::models::ProjectPermission::BucketsAppend,
            "files-write" => openapi::models::ProjectPermission::FilesWrite,
            "appliance-config-write" => openapi::models::ProjectPermission::ApplianceConfigWrite,
            "appliance-config-read" => openapi::models::ProjectPermission::ApplianceConfigRead,
            "integrations-write" => openapi::models::ProjectPermission::IntegrationsWrite,
            "integrations-read" => openapi::models::ProjectPermission::IntegrationsRead,
            "alerts-read" => openapi::models::ProjectPermission::AlertsRead,
            "alerts-write" => openapi::models::ProjectPermission::AlertsWrite,
            "role-cm-project-admin" => openapi::models::ProjectPermission::RoleCmProjectAdmin,
            _ => return Err(anyhow::anyhow!("Invalid project permission: {}", s)),
        };
        Ok(ProjectPermission(perm))
    }
}

impl From<ProjectPermission> for openapi::models::ProjectPermission {
    fn from(perm: ProjectPermission) -> Self {
        perm.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_permission_from_str() {
        let perm = GlobalPermission::from_str("root").unwrap();
        assert_eq!(perm.0, openapi::models::GlobalPermission::Root);

        let perm = GlobalPermission::from_str("tenant-admin").unwrap();
        assert_eq!(perm.0, openapi::models::GlobalPermission::TenantAdmin);
    }

    #[test]
    fn test_project_permission_from_str() {
        let perm = ProjectPermission::from_str("sources-admin").unwrap();
        assert_eq!(perm.0, openapi::models::ProjectPermission::SourcesAdmin);

        let perm = ProjectPermission::from_str("datasets-admin").unwrap();
        assert_eq!(perm.0, openapi::models::ProjectPermission::DatasetsAdmin);
    }
}
