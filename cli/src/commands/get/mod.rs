// Module declarations
mod audit_events;
mod buckets;
pub mod comments;
mod custom_label_trend_report;
mod datasets;
mod emails;
mod integrations;
mod keyed_sync_states;
mod projects;
mod quota;
mod sources;
mod streams;
mod users;

use anyhow::Result;
use scoped_threadpool::Pool;
use structopt::StructOpt;

use openapi::apis::configuration::Configuration;

use self::{
    audit_events::GetAuditEventsArgs,
    buckets::GetBucketsArgs,
    comments::{GetManyCommentsArgs, GetSingleCommentArgs},
    custom_label_trend_report::GetCustomLabelTrendReportArgs,
    datasets::GetDatasetsArgs,
    emails::GetManyEmailsArgs,
    integrations::GetIntegrationsArgs,
    keyed_sync_states::GetKeyedSyncStatesArgs,
    projects::GetProjectsArgs,
    quota::GetQuotaArgs,
    sources::GetSourcesArgs,
    streams::{GetStreamCommentsArgs, GetStreamStatsArgs, GetStreamsArgs},
    users::GetUsersArgs,
};
use crate::printer::Printer;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, StructOpt)]
pub enum GetArgs {
    #[structopt(name = "buckets")]
    /// List the available buckets
    Buckets(GetBucketsArgs),

    #[structopt(name = "emails")]
    /// Download all emails from a source
    Emails(GetManyEmailsArgs),

    #[structopt(name = "comment")]
    /// Get a single comment from a source
    Comment(GetSingleCommentArgs),

    #[structopt(name = "comments")]
    /// Download all comments from a source
    Comments(GetManyCommentsArgs),

    #[structopt(name = "datasets")]
    /// List the available datasets
    Datasets(GetDatasetsArgs),

    #[structopt(name = "projects")]
    /// List the available projects
    Projects(GetProjectsArgs),

    #[structopt(name = "sources")]
    /// List the available sources
    Sources(GetSourcesArgs),

    #[structopt(name = "streams")]
    /// List the available streams for a dataset
    Streams(GetStreamsArgs),

    #[structopt(name = "stream-comments")]
    /// Fetch comments from a stream
    StreamComments(GetStreamCommentsArgs),

    #[structopt(name = "stream-stats")]
    /// Get the validation stats for a given stream
    StreamStats(GetStreamStatsArgs),

    #[structopt(name = "users")]
    /// List the available users
    Users(GetUsersArgs),

    #[structopt(name = "current-user")]
    /// Get the user associated with the API token in use
    CurrentUser,

    #[structopt(name = "quotas")]
    /// List all quotas for current tenant
    Quotas(GetQuotaArgs),

    #[structopt(name = "audit-events")]
    /// Get audit events for current tenant
    AuditEvents(GetAuditEventsArgs),

    #[structopt(name = "integrations")]
    /// Get integrations
    Integrations(GetIntegrationsArgs),

    #[structopt(name = "keyed-sync-states")]
    /// Get keyed sync states
    KeyedSyncStates(GetKeyedSyncStatesArgs),

    #[structopt(name = "custom-label-trend-report")]
    /// Get Custom Report
    CustomDatasetReport(GetCustomLabelTrendReportArgs),
}

/// Execute get commands based on the provided arguments
pub fn run(
    args: &GetArgs,
    config: &Configuration,
    printer: &Printer,
    pool: &mut Pool,
) -> Result<()> {
    match args {
        GetArgs::Buckets(args) => buckets::get(config, args, printer),
        GetArgs::Emails(args) => emails::get_many(config, args),
        GetArgs::Comment(args) => comments::get_single(config, args),
        GetArgs::Comments(args) => comments::get_many(config, args),
        GetArgs::Datasets(args) => datasets::get(config, args, printer, pool),
        GetArgs::Projects(args) => projects::get(config, args, printer),
        GetArgs::Sources(args) => sources::get(config, args, printer),
        GetArgs::Streams(args) => streams::get(config, args, printer),
        GetArgs::StreamComments(args) => streams::get_stream_comments(config, args),
        GetArgs::StreamStats(args) => streams::get_stream_stats(config, args, printer, pool),
        GetArgs::Users(args) => users::get(config, args, printer),
        GetArgs::CurrentUser => users::get_current_user_and_print(config, printer),
        GetArgs::Quotas(args) => quota::get(config, args, printer),
        GetArgs::AuditEvents(args) => audit_events::get(config, args, printer),
        GetArgs::Integrations(args) => integrations::get(config, args, printer),
        GetArgs::KeyedSyncStates(args) => keyed_sync_states::get(config, args, printer),
        GetArgs::CustomDatasetReport(args) => custom_label_trend_report::get(config, args, printer),
    }
}
