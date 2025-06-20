use crate::printer::Printer;
use anyhow::Result;
use reinfer_client::{resources::tenant_id::UiPathTenantId, Client};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GetQuotaArgs {
    #[structopt(short = "t", long = "uipath-tenant-id")]
    /// The tenant id to get quotas for, if not provided the current tenant is used
    tenant_id: Option<UiPathTenantId>,
}

pub fn get(client: &Client, args: &GetQuotaArgs, printer: &Printer) -> Result<()> {
    let GetQuotaArgs { tenant_id } = args;

    printer.print_resources(&client.get_quotas(tenant_id)?)
}
