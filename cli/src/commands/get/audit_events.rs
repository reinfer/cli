use anyhow::Result;
use chrono::{DateTime, Utc};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        audit_events_api::query_audit_events,
    },
    models::{QueryAuditEventsRequest, QueryAuditEventsFilter, TimestampRangeFilter},
};
use structopt::StructOpt;

use crate::printer::Printer;
use crate::utils::AuditEventsResponseExt;

#[derive(Debug, StructOpt)]
pub struct GetAuditEventsArgs {
    #[structopt(short = "m", long = "minimum")]
    /// Minimum Timestamp for audit events
    minimum_timestamp: Option<DateTime<Utc>>,

    #[structopt(short = "M", long = "maximum")]
    /// Maximum Timestamp for audit events
    maximum_timestamp: Option<DateTime<Utc>>,
}

pub fn get(config: &Configuration, args: &GetAuditEventsArgs, printer: &Printer) -> Result<()> {
    let GetAuditEventsArgs {
        minimum_timestamp,
        maximum_timestamp,
    } = args;

    let mut continuation = None;
    let mut all_printable_events = Vec::new();

    loop {
        // Build the timestamp filter if timestamps are provided
        let timestamp_filter = if minimum_timestamp.is_some() || maximum_timestamp.is_some() {
            Some(Box::new(TimestampRangeFilter {
                minimum: minimum_timestamp.map(|ts| ts.to_rfc3339()),
                maximum: maximum_timestamp.map(|ts| ts.to_rfc3339()),
            }))
        } else {
            None
        };

        let filter = if timestamp_filter.is_some() {
            Some(Box::new(QueryAuditEventsFilter {
                timestamp: timestamp_filter,
            }))
        } else {
            None
        };

        let request = QueryAuditEventsRequest {
            limit: None,
            continuation: continuation.clone(),
            filter,
        };

        let response = query_audit_events(config, request)?;
        
        // Check continuation before moving response
        let has_continuation = response.continuation.is_some();
        continuation = response.continuation.clone();
        
        // Use the clean iterator interface - same as legacy API!
        let mut printable_events: Vec<_> = response.into_iter_printable().collect();
        all_printable_events.append(&mut printable_events);

        if !has_continuation {
            break;
        } else {
            info!("Downloaded {} events", all_printable_events.len());
        }
    }

    printer.print_resources(&all_printable_events)
}
