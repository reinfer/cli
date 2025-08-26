//! Commands for managing tenant quotas
//!
//! This module provides functionality to:
//! - Set quotas for Reinfer and UiPath tenants
//! - Handle quota kind validation and parsing
//! - Support auto-increase limits for quotas
//! - Provide comprehensive error handling

// External crate imports
use anyhow::{anyhow, Context, Result};
use log::info;
use structopt::{clap::ArgGroup, StructOpt};

// OpenAPI imports
use openapi::{
    apis::{configuration::Configuration, quotas_api::set_quota_for_tenant},
    models::{QuotaKind, SetQuotaForTenantRequest},
};

/// Command line arguments for setting tenant quotas
#[derive(Debug, StructOpt)]
#[structopt(group = ArgGroup::with_name("tenant-id").required(true))]
pub struct CreateQuotaArgs {
    #[structopt(long = "reinfer-tenant-id", group = "tenant-id")]
    /// Reinfer tenant ID for which to set the quota
    reinfer_tenant_id: Option<String>,

    #[structopt(long = "uipath-tenant-id", group = "tenant-id")]
    /// UiPath tenant ID for which to set the quota
    uipath_tenant_id: Option<String>,

    #[structopt(long = "quota-kind")]
    /// Kind of quota to set
    tenant_quota_kind: String,

    #[structopt(long = "limit")]
    /// New value of the quota to set
    hard_limit: i32,

    #[structopt(long = "auto-increase-up-to")]
    /// If set, will also change the `auto-increase-up-to` value of the quota
    auto_increase_up_to: Option<i32>,
}

/// Set a quota value for a tenant with optional auto-increase limit
///
/// This function handles:
/// - Tenant ID validation (Reinfer or UiPath)
/// - Quota kind parsing and validation
/// - API call to set the quota value
pub fn create(config: &Configuration, args: &CreateQuotaArgs) -> Result<()> {
    let CreateQuotaArgs {
        reinfer_tenant_id,
        uipath_tenant_id,
        tenant_quota_kind,
        hard_limit,
        auto_increase_up_to,
    } = args;

    let tenant_id = resolve_tenant_id(reinfer_tenant_id, uipath_tenant_id)?;

    let quota_kind = parse_quota_kind(tenant_quota_kind)?;

    set_tenant_quota(
        config,
        &tenant_id,
        &quota_kind,
        *hard_limit,
        *auto_increase_up_to,
    )?;

    info!(
        "New quota `{}` set successfully in tenant with id `{}`",
        tenant_quota_kind, tenant_id
    );

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Resolve and validate tenant ID from the provided options
fn resolve_tenant_id(
    reinfer_tenant_id: &Option<String>,
    uipath_tenant_id: &Option<String>,
) -> Result<String> {
    match (reinfer_tenant_id, uipath_tenant_id) {
        (Some(tenant_id), None) => Ok(tenant_id.clone()),
        (None, Some(tenant_id)) => Ok(tenant_id.clone()),
        _ => Err(anyhow!(
            "Expected one and only one tenant ID, got none or two."
        )),
    }
}

/// Parse a quota kind string into the corresponding enum variant
fn parse_quota_kind(quota_kind_str: &str) -> Result<QuotaKind> {
    match quota_kind_str {
        "alerts" => Ok(QuotaKind::Alerts),
        "buckets" => Ok(QuotaKind::Buckets),
        "comments" => Ok(QuotaKind::Comments),
        "comments_per_source" => Ok(QuotaKind::CommentsPerSource),
        "comments_in_ixp_runtime" => Ok(QuotaKind::CommentsInIxpRuntime),
        "comments_in_ixp_designtime" => Ok(QuotaKind::CommentsInIxpDesigntime),
        "datasets" => Ok(QuotaKind::Datasets),
        "datasets_per_source" => Ok(QuotaKind::DatasetsPerSource),
        "entities_per_dataset" => Ok(QuotaKind::EntitiesPerDataset),
        "integrations" => Ok(QuotaKind::Integrations),
        "labels_per_dataset" => Ok(QuotaKind::LabelsPerDataset),
        "mailboxes_per_integration" => Ok(QuotaKind::MailboxesPerIntegration),
        "pinned_models" => Ok(QuotaKind::PinnedModels),
        "projects" => Ok(QuotaKind::Projects),
        "reviewed_comments_per_dataset" => Ok(QuotaKind::ReviewedCommentsPerDataset),
        "sources" => Ok(QuotaKind::Sources),
        "sources_per_dataset" => Ok(QuotaKind::SourcesPerDataset),
        "triggers" => Ok(QuotaKind::Triggers),
        "triggers_per_dataset" => Ok(QuotaKind::TriggersPerDataset),
        "users" => Ok(QuotaKind::Users),
        "extraction_predictions" => Ok(QuotaKind::ExtractionPredictions),
        _ => Err(anyhow!("Invalid quota kind: '{}'", quota_kind_str)),
    }
}

/// Set a quota for a tenant via API call
fn set_tenant_quota(
    config: &Configuration,
    tenant_id: &str,
    quota_kind: &QuotaKind,
    hard_limit: i32,
    auto_increase_up_to: Option<i32>,
) -> Result<()> {
    let request = SetQuotaForTenantRequest {
        hard_limit,
        auto_increase_up_to: auto_increase_up_to.map(Some),
    };

    set_quota_for_tenant(config, tenant_id, &quota_kind.to_string(), request)
        .context("Operation to set quota has failed")?;

    Ok(())
}
