use anyhow::{Context, Result};
use structopt::StructOpt;

use openapi::{
    apis::{
        configuration::Configuration,
        quotas_api::{get_quotas_for_tenant, get_tenant_quota},
    },
    models::Quota,
};

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetQuotaArgs {
    #[structopt(short = "t", long = "uipath-tenant-id")]
    /// The tenant id to get quotas for, if not provided the current tenant is used
    tenant_id: Option<String>,
}

/// Retrieve quotas for the current tenant or a specific tenant
pub fn get(config: &Configuration, args: &GetQuotaArgs, printer: &Printer) -> Result<()> {
    let GetQuotaArgs { tenant_id } = args;

    let quotas: Vec<Quota> = if let Some(tenant_id) = tenant_id {
        get_quotas_for_tenant(config, tenant_id)
            .context("Failed to get quotas for tenant")?
            .quotas
    } else {
        get_tenant_quota(config)
            .context("Failed to get tenant quotas")?
            .quotas
    };

    printer.print_resources(&quotas)
}
