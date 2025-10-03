use anyhow::{anyhow, Context, Result};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        quotas_api::set_quota_for_tenant,
    },
    models::{QuotaKind, SetQuotaForTenantRequest},
};
use structopt::{clap::ArgGroup, StructOpt};

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

pub fn create(config: &Configuration, args: &CreateQuotaArgs) -> Result<()> {
    let CreateQuotaArgs {
        reinfer_tenant_id,
        uipath_tenant_id,
        tenant_quota_kind,
        hard_limit,
        auto_increase_up_to,
    } = args;

    // Determine tenant ID (same logic as legacy version)
    let tenant_id = match (reinfer_tenant_id, uipath_tenant_id) {
        (Some(tenant_id), None) => tenant_id,
        (None, Some(tenant_id)) => tenant_id,
        _ => {
            return Err(anyhow!(
                "Expected one and only one tenant ID, got none or two."
            ))
        }
    };

    // Parse the quota kind string
    let quota_kind = match tenant_quota_kind.as_str() {
        "alerts" => QuotaKind::Alerts,
        "buckets" => QuotaKind::Buckets,
        "comments" => QuotaKind::Comments,
        "comments_per_source" => QuotaKind::CommentsPerSource,
        "comments_in_ixp_runtime" => QuotaKind::CommentsInIxpRuntime,
        "comments_in_ixp_designtime" => QuotaKind::CommentsInIxpDesigntime,
        "datasets" => QuotaKind::Datasets,
        "datasets_per_source" => QuotaKind::DatasetsPerSource,
        "entities_per_dataset" => QuotaKind::EntitiesPerDataset,
        "integrations" => QuotaKind::Integrations,
        "labels_per_dataset" => QuotaKind::LabelsPerDataset,
        "mailboxes_per_integration" => QuotaKind::MailboxesPerIntegration,
        "pinned_models" => QuotaKind::PinnedModels,
        "projects" => QuotaKind::Projects,
        "reviewed_comments_per_dataset" => QuotaKind::ReviewedCommentsPerDataset,
        "sources" => QuotaKind::Sources,
        "sources_per_dataset" => QuotaKind::SourcesPerDataset,
        "triggers" => QuotaKind::Triggers,
        "triggers_per_dataset" => QuotaKind::TriggersPerDataset,
        "users" => QuotaKind::Users,
        "extraction_predictions" => QuotaKind::ExtractionPredictions,
        _ => return Err(anyhow!("Invalid quota kind: {}", tenant_quota_kind)),
    };

    // Single API call (same for both Reinfer and UiPath tenant IDs)
    let request = SetQuotaForTenantRequest {
        hard_limit: *hard_limit,
        auto_increase_up_to: auto_increase_up_to.map(Some),
    };
    
    set_quota_for_tenant(config, tenant_id, &quota_kind.to_string(), request)
        .context("Operation to set quota has failed")?;
    
    info!("New quota `{}` set successfully in tenant with id `{}`", tenant_quota_kind, tenant_id);

    Ok(())
}
