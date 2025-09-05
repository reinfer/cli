use anyhow::{anyhow, Context, Result};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        quotas_api::{set_quota_for_tenant, set_tenant_quota},
    },
    models::{QuotaKind as TenantQuotaKind, Tenant, SetQuotaForTenantRequest},
};
use structopt::{clap::ArgGroup, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(group = ArgGroup::with_name("tenant-id").required(true))]
pub struct CreateQuotaArgs {
    #[structopt(long = "reinfer-tenant-id", group = "tenant-id")]
    /// Reinfer tenant ID for which to set the quota
    reinfer_tenant_id: Option<Tenant>,

    #[structopt(long = "uipath-tenant-id", group = "tenant-id")]
    /// UiPath tenant ID for which to set the quota
    uipath_tenant_id: Option<Tenant>,

    #[structopt(long = "quota-kind")]
    /// Kind of quota to set
    tenant_quota_kind: TenantQuotaKind,

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
        (Some(tenant), None) => &tenant.tenant_id,
        (None, Some(tenant)) => &tenant.tenant_id,
        _ => {
            return Err(anyhow!(
                "Expected one and only one tenant ID, got none or two."
            ))
        }
    };

    let quota_kind_str = tenant_quota_kind.to_string();

    // Single API call (same for both Reinfer and UiPath tenant IDs)
    let request = SetQuotaForTenantRequest {
        hard_limit: *hard_limit,
        auto_increase_up_to: auto_increase_up_to.map(Some),
    };
    
    set_quota_for_tenant(config, tenant_id, &quota_kind_str, request)
        .context("Operation to set quota has failed")?;
    
    info!("New quota `{}` set successfully in tenant with id `{}`", quota_kind_str, tenant_id);

    Ok(())
}
