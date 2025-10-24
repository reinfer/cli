use anyhow::Result;
use chrono::{DateTime, Utc};
use log::info;
use structopt::StructOpt;

use openapi::{
    apis::{audit_events_api::query_audit_events, configuration::Configuration},
    models::{QueryAuditEventsFilter, QueryAuditEventsRequest, TimestampRangeFilter},
};

use crate::{printer::Printer, utils::AuditEventsResponseExt};

#[derive(Debug, StructOpt)]
pub struct GetAuditEventsArgs {
    #[structopt(short = "m", long = "minimum")]
    /// Minimum timestamp for audit events
    minimum_timestamp: Option<DateTime<Utc>>,

    #[structopt(short = "M", long = "maximum")]
    /// Maximum timestamp for audit events
    maximum_timestamp: Option<DateTime<Utc>>,
}

/// Retrieve audit events with optional timestamp filtering
pub fn get(config: &Configuration, args: &GetAuditEventsArgs, printer: &Printer) -> Result<()> {
    let GetAuditEventsArgs {
        minimum_timestamp,
        maximum_timestamp,
    } = args;

    let filter = build_timestamp_filter(*minimum_timestamp, *maximum_timestamp);
    let all_events = fetch_all_audit_events(config, filter)?;

    printer.print_resources(&all_events)
}

/// Build timestamp filter if any timestamps are provided
fn build_timestamp_filter(
    minimum: Option<DateTime<Utc>>,
    maximum: Option<DateTime<Utc>>,
) -> Option<Box<QueryAuditEventsFilter>> {
    if minimum.is_none() && maximum.is_none() {
        return None;
    }

    let timestamp_filter = Box::new(TimestampRangeFilter {
        minimum: minimum.map(|ts| ts.to_rfc3339()),
        maximum: maximum.map(|ts| ts.to_rfc3339()),
    });

    Some(Box::new(QueryAuditEventsFilter {
        timestamp: Some(timestamp_filter),
    }))
}

/// Fetch all audit events using pagination
fn fetch_all_audit_events(
    config: &Configuration,
    filter: Option<Box<QueryAuditEventsFilter>>,
) -> Result<Vec<crate::printer::PrintableAuditEvent>> {
    let mut continuation = None;
    let mut all_events = Vec::new();

    loop {
        let request = QueryAuditEventsRequest {
            limit: None,
            continuation: continuation.clone(),
            filter: filter.clone(),
        };

        let response = query_audit_events(config, request)?;
        let has_continuation = response.continuation.is_some();
        continuation = response.continuation.clone();

        let mut events: Vec<_> = response.into_iter_printable().collect();
        all_events.append(&mut events);

        if !has_continuation {
            break;
        }

        info!("Downloaded {} events", all_events.len());
    }

    Ok(all_events)
}
