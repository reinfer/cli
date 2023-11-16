use anyhow::Result;
use chrono::{DateTime, Utc};
use log::info;
use reinfer_client::{resources::audit::PrintableAuditEvent, Client};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetAuditEventsArgs {
    #[structopt(short = "m", long = "minimum")]
    /// Minimum Timestamp for audit events
    minimum_timestamp: Option<DateTime<Utc>>,

    #[structopt(short = "M", long = "maximum")]
    /// Maximum Timestamp for audit events
    maximum_timestamp: Option<DateTime<Utc>>,
}

pub fn get(client: &Client, args: &GetAuditEventsArgs, printer: &Printer) -> Result<()> {
    let GetAuditEventsArgs {
        minimum_timestamp,
        maximum_timestamp,
    } = args;

    let mut continuation = None;

    let mut all_printable_events = Vec::new();

    loop {
        let audit_events =
            client.get_audit_events(*minimum_timestamp, *maximum_timestamp, continuation)?;
        let mut printable_events: Vec<PrintableAuditEvent> =
            audit_events.clone().into_iter_printable().collect();

        all_printable_events.append(&mut printable_events);

        if audit_events.continuation.is_none() {
            break;
        } else {
            info!("Downloaded {} events", all_printable_events.len());
            continuation = audit_events.continuation
        }
    }

    printer.print_resources(all_printable_events.iter())
}
