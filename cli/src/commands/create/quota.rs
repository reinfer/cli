use anyhow::{anyhow, Context, Result};
use log::info;
use reinfer_client::{
    resources::{
        quota::{CreateQuota, TenantQuotaKind},
        tenant_id::{ReinferTenantId, TenantId, UiPathTenantId},
    },
    Client,
};
use structopt::{clap::ArgGroup, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(group = ArgGroup::with_name("tenant-id").required(true))]
pub struct CreateQuotaArgs {
    #[structopt(long = "reinfer-tenant-id", group = "tenant-id")]
    /// Reinfer tenant ID for which to set the quota
    reinfer_tenant_id: Option<ReinferTenantId>,

    #[structopt(long = "uipath-tenant-id", group = "tenant-id")]
    /// UiPath tenant ID for which to set the quota
    uipath_tenant_id: Option<UiPathTenantId>,

    #[structopt(long = "quota-kind")]
    /// Kind of quota to set
    tenant_quota_kind: TenantQuotaKind,

    #[structopt(long = "limit")]
    /// New value of the quota to set
    hard_limit: u32,
}

pub fn create(client: &Client, args: &CreateQuotaArgs) -> Result<()> {
    let CreateQuotaArgs {
        reinfer_tenant_id,
        uipath_tenant_id,
        tenant_quota_kind,
        hard_limit,
    } = args;

    let tenant_id: TenantId = match (reinfer_tenant_id, uipath_tenant_id) {
        (Some(tenant_id), None) => TenantId::Reinfer(tenant_id.to_owned()),
        (None, Some(tenant_id)) => TenantId::UiPath(tenant_id.to_owned()),
        _ => {
            return Err(anyhow!(
                "Expected one and only one tenant ID, got none or two."
            ))
        }
    };

    client
        .create_quota(
            &tenant_id,
            *tenant_quota_kind,
            CreateQuota {
                hard_limit: *hard_limit,
            },
        )
        .context("Operation to set quota has failed")?;

    info!(
        "New quota `{}` set successfully in tenant with id `{}`",
        tenant_quota_kind, tenant_id
    );
    Ok(())
}
