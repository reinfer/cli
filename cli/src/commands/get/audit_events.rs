use anyhow::Result;
use chrono::{DateTime, Utc};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        audit_events_api::query_audit_events,
    },
    models::{AuditEvent, QueryAuditEventsRequest, QueryAuditEventsFilter, TimestampRangeFilter},
};
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

pub fn get(config: &Configuration, args: &GetAuditEventsArgs, printer: &Printer) -> Result<()> {
    let GetAuditEventsArgs {
        minimum_timestamp,
        maximum_timestamp,
    } = args;

    let mut continuation = None;
    let mut all_audit_events = Vec::new();

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
        
        all_audit_events.extend(response.audit_events);

        if response.continuation.is_none() {
            break;
        } else {
            info!("Downloaded {} events", all_audit_events.len());
            continuation = response.continuation;
        }
    }

    printer.print_resources(all_audit_events.iter())
}
