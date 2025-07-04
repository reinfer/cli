use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, SerializeDisplay, DeserializeFromStr, PartialEq, Eq, Hash, Copy)]
pub enum TenantQuotaKind {
    Sources,
    SourcesPerDataset,
    Datasets,
    DatasetsPerSource,
    LabelsPerDataset,
    EntitiesPerDataset,
    Comments,
    CommentsPerSource,
    CommentsInIxpDesigntime,
    CommentsInIxpRuntime,
    ReviewedCommentsPerDataset,
    Integrations,
    MailboxesPerIntegration,
    Triggers,
    TriggersPerDataset,
    Users,
    Alerts,
    Buckets,
    Projects,
    PinnedModels,
    ExtractionPredictions,
}

impl FromStr for TenantQuotaKind {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        let tenant_quota_kind = match string {
            "sources" => TenantQuotaKind::Sources,
            "sources_per_dataset" => TenantQuotaKind::SourcesPerDataset,
            "datasets" => TenantQuotaKind::Datasets,
            "datasets_per_source" => TenantQuotaKind::DatasetsPerSource,
            "labels_per_dataset" => TenantQuotaKind::LabelsPerDataset,
            "entities_per_dataset" => TenantQuotaKind::EntitiesPerDataset,
            "comments" => TenantQuotaKind::Comments,
            "comments_per_source" => TenantQuotaKind::CommentsPerSource,
            "comments_in_ixp_designtime" => TenantQuotaKind::CommentsInIxpDesigntime,
            "comments_in_ixp_runtime" => TenantQuotaKind::CommentsInIxpRuntime,
            "reviewed_comments_per_dataset" => TenantQuotaKind::ReviewedCommentsPerDataset,
            "integrations" => TenantQuotaKind::Integrations,
            "mailboxes_per_integration" => TenantQuotaKind::MailboxesPerIntegration,
            "triggers" => TenantQuotaKind::Triggers,
            "triggers_per_dataset" => TenantQuotaKind::TriggersPerDataset,
            "users" => TenantQuotaKind::Users,
            "alerts" => TenantQuotaKind::Alerts,
            "buckets" => TenantQuotaKind::Buckets,
            "projects" => TenantQuotaKind::Projects,
            "pinned_models" => TenantQuotaKind::PinnedModels,
            "extraction_predictions" => TenantQuotaKind::ExtractionPredictions,
            _ => {
                return Err(Error::BadTenantQuotaKind {
                    tenant_quota_kind: string.to_string(),
                })
            }
        };
        Ok(tenant_quota_kind)
    }
}

impl Display for TenantQuotaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TenantQuotaKind::Sources => "sources",
                TenantQuotaKind::SourcesPerDataset => "sources_per_dataset",
                TenantQuotaKind::Datasets => "datasets",
                TenantQuotaKind::DatasetsPerSource => "datasets_per_source",
                TenantQuotaKind::LabelsPerDataset => "labels_per_dataset",
                TenantQuotaKind::EntitiesPerDataset => "entities_per_dataset",
                TenantQuotaKind::Comments => "comments",
                TenantQuotaKind::CommentsPerSource => "comments_per_source",
                TenantQuotaKind::CommentsInIxpDesigntime => "comments_in_ixp_designtime",

                TenantQuotaKind::CommentsInIxpRuntime => "comments_in_ixp_runtime",
                TenantQuotaKind::ReviewedCommentsPerDataset => "reviewed_comments_per_dataset",
                TenantQuotaKind::Integrations => "integrations",
                TenantQuotaKind::MailboxesPerIntegration => "mailboxes_per_integration",
                TenantQuotaKind::Triggers => "triggers",
                TenantQuotaKind::TriggersPerDataset => "triggers_per_dataset",
                TenantQuotaKind::Users => "users",
                TenantQuotaKind::Alerts => "alerts",
                TenantQuotaKind::Buckets => "buckets",
                TenantQuotaKind::Projects => "projects",
                TenantQuotaKind::PinnedModels => "pinned_models",
                TenantQuotaKind::ExtractionPredictions => "extraction_predictions",
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct CreateQuota {
    pub hard_limit: u64,

    // It is very important for this value not to be serialized if it is `None`.
    // This is because the API will interpret `null` as "reset the auto-increase-up-to" value to
    // its default value. If the field is not set, then the value will be left unchanged (which is
    // what we want).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_increase_up_to: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub struct Quota {
    pub hard_limit: u64,
    pub quota_kind: TenantQuotaKind,
    pub current_max_usage: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetQuotasResponse {
    pub quotas: Vec<Quota>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use matches::assert_matches;

    #[test]
    fn tenant_quota_kind_roundtrips() {
        let kinds = vec![
            ("sources", TenantQuotaKind::Sources),
            ("sources_per_dataset", TenantQuotaKind::SourcesPerDataset),
            ("datasets", TenantQuotaKind::Datasets),
            ("datasets_per_source", TenantQuotaKind::DatasetsPerSource),
            ("labels_per_dataset", TenantQuotaKind::LabelsPerDataset),
            ("entities_per_dataset", TenantQuotaKind::EntitiesPerDataset),
            ("comments", TenantQuotaKind::Comments),
            ("comments_per_source", TenantQuotaKind::CommentsPerSource),
            (
                "comments_in_ixp_designtime",
                TenantQuotaKind::CommentsInIxpDesigntime,
            ),
            (
                "comments_in_ixp_runtime",
                TenantQuotaKind::CommentsInIxpRuntime,
            ),
            (
                "reviewed_comments_per_dataset",
                TenantQuotaKind::ReviewedCommentsPerDataset,
            ),
            ("integrations", TenantQuotaKind::Integrations),
            (
                "mailboxes_per_integration",
                TenantQuotaKind::MailboxesPerIntegration,
            ),
            ("triggers", TenantQuotaKind::Triggers),
            ("triggers_per_dataset", TenantQuotaKind::TriggersPerDataset),
            ("users", TenantQuotaKind::Users),
            ("alerts", TenantQuotaKind::Alerts),
            ("buckets", TenantQuotaKind::Buckets),
            ("projects", TenantQuotaKind::Projects),
            ("pinned_models", TenantQuotaKind::PinnedModels),
            (
                "extraction_predictions",
                TenantQuotaKind::ExtractionPredictions,
            ),
        ];

        for (string, kind) in kinds {
            assert_eq!(TenantQuotaKind::from_str(string).unwrap(), kind);
            assert_eq!(
                &serde_json::ser::to_string(&kind).unwrap(),
                &format!("\"{string}\"")
            );
        }
    }

    #[test]
    fn deserialising_unknown_tenant_quota_kind_fails() {
        assert_matches!(
            TenantQuotaKind::from_str("unknown"),
            Err(Error::BadTenantQuotaKind { .. })
        );
    }
}
