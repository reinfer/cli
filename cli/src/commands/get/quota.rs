use crate::printer::Printer;
use anyhow::Result;
use reinfer_client::Client;

pub fn get(client: &Client, printer: &Printer) -> Result<()> {
    printer.print_resources(&client.get_quotas()?)
}
