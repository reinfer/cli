use super::thousands::Thousands;
use colored::Colorize;
use openapi::models::{
    AuditEvent, Bucket, BucketStatistics, Count, Dataset, Integration, KeyedSyncState,
    ListKeyedSyncStatesResponseKeyedSyncStatesInner, Project, Quota, Source, SourceStatistics,
    Statistics, Trigger, User,
};
use prettytable::{format, row, Row, Table};
use serde::{Serialize, Serializer};

use anyhow::{anyhow, Context, Error, Result};
use std::{
    io::{self, Write},
    str::FromStr,
};

pub fn print_resources_as_json<Resource>(
    resources: impl IntoIterator<Item = Resource>,
    mut writer: impl Write,
) -> Result<()>
where
    Resource: Serialize,
{
    for resource in resources {
        serde_json::to_writer(&mut writer, &resource)
            .context("Could not serialise resource.")
            .and_then(|_| writeln!(writer).context("Failed to write JSON resource to writer."))?;
    }
    Ok(())
}

#[derive(Copy, Clone, Default, Debug)]
pub enum OutputFormat {
    Json,
    #[default]
    Table,
}

impl FromStr for OutputFormat {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string == "table" {
            Ok(OutputFormat::Table)
        } else if string == "json" {
            Ok(OutputFormat::Json)
        } else {
            Err(anyhow!("{}", string))
        }
    }
}

/// Represents a resource that is able to be displayed as a table.
///
/// The implementation must implement `to_table_headers` to return headers for the resource type,
/// and `to_table_row`, which should return a data row for the given resource instance.
pub trait DisplayTable {
    fn to_table_headers() -> Row;

    fn to_table_row(&self) -> Row;
}

impl DisplayTable for Integration {
    fn to_table_headers() -> Row {
        row![bFg => "Project", "Name", "ID", "Created (UTC)", "Mailbox Count"]
    }

    fn to_table_row(&self) -> Row {
        // Parse the created_at timestamp
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .unwrap_or_else(|_| {
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
            })
            .format("%Y-%m-%d %H:%M:%S");

        // Extract mailbox count from configuration JSON
        let mailbox_count = self
            .configuration
            .get("mailboxes")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        row![self.owner, self.name, self.id, created_at, mailbox_count]
    }
}
impl DisplayTable for Bucket {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Created (UTC)"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!("{}{}{}", self.owner.dimmed(), "/".dimmed(), self.name);
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .unwrap_or_else(|_| {
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
            })
            .format("%Y-%m-%d %H:%M:%S");
        row![full_name, self.id, created_at,]
    }
}

impl DisplayTable for Quota {
    fn to_table_headers() -> Row {
        row![bFg => "Kind", "Hard Limit", "Usage (Total)", "Usage %"]
    }

    fn to_table_row(&self) -> Row {
        row![
            self.quota_kind,
            Thousands(self.hard_limit as u64),
            Thousands(self.current_max_usage as u64),
            if self.hard_limit > 0 {
                format!(
                    "{:.0}%",
                    (self.current_max_usage as f64 / self.hard_limit as f64) * 100.0
                )
            } else {
                "N/A".dimmed().to_string()
            }
        ]
    }
}

impl DisplayTable for Dataset {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Title"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!("{}{}{}", self.owner.dimmed(), "/".dimmed(), self.name);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.last_modified)
            .unwrap_or_else(|_| {
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
            })
            .format("%Y-%m-%d %H:%M:%S");
        row![full_name, self.id, updated_at, self.title,]
    }
}

// Custom struct for dataset with statistics (OpenAPI equivalent)
#[derive(Debug)]
pub struct PrintableDatasetWithStats {
    pub dataset: Dataset,
    pub stats: Option<Statistics>,
    pub validation: Option<openapi::models::GetValidationResponse>,
}

impl DisplayTable for PrintableDatasetWithStats {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Title", "Total Comments", "Validation"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!(
            "{}{}{}",
            self.dataset.owner.dimmed(),
            "/".dimmed(),
            self.dataset.name
        );

        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.dataset.last_modified)
            .unwrap_or_else(|_| {
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
            })
            .format("%Y-%m-%d %H:%M:%S");

        let comment_count = if let Some(stats) = &self.stats {
            stats.num_comments as i64
        } else {
            0
        };

        let validation_status = if let Some(validation) = &self.validation {
            // Show validation data is available with model score
            let rating = &validation.validation.model_rating;
            let quality_score = rating.score * 100.0;
            format!(
                "✓ {:.1}% ({} labels)",
                format!("{:.1}", quality_score).green(),
                validation.validation.labels.len()
            )
        } else {
            "No validation".dimmed().to_string()
        };

        row![
            full_name,
            self.dataset.id,
            updated_at,
            self.dataset.title,
            Thousands(comment_count as u64),
            validation_status,
        ]
    }
}

impl Serialize for PrintableDatasetWithStats {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.dataset, serializer)
    }
}

impl DisplayTable for Project {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Title"]
    }

    fn to_table_row(&self) -> Row {
        row![self.name, self.id, self.title]
    }
}

impl DisplayTable for Source {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Transform Tag", "Title", "Bucket"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!("{}{}{}", self.owner.dimmed(), "/".dimmed(), self.name);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .unwrap_or_else(|_| {
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
            })
            .format("%Y-%m-%d %H:%M:%S");
        row![
            full_name,
            self.id,
            updated_at,
            match &self.email_transform_tag {
                Some(transform_tag) => transform_tag.as_str().into(),
                None => "missing".dimmed(),
            },
            self.title,
            match &self.bucket_id {
                Some(bucket) => bucket.as_str().into(),
                None => "missing".dimmed(),
            }
        ]
    }
}

#[derive(Debug)]
pub struct PrintableBucket {
    pub bucket: Bucket,
    pub stats: Option<BucketStatistics>,
}
impl DisplayTable for PrintableBucket {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Created (UTC)", "Num Emails"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!(
            "{}{}{}",
            self.bucket.owner.dimmed(),
            "/".dimmed(),
            self.bucket.name
        );
        let count_str = if let Some(stats) = &self.stats {
            match &*stats.count {
                Count::LowerBound(count) => format!(">={}", count.value),
                Count::Exact(count) => format!("={}", count.value),
            }
        } else {
            "none".dimmed().to_string()
        };
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.bucket.created_at)
            .unwrap_or_else(|_| {
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
            })
            .format("%Y-%m-%d %H:%M:%S");
        row![full_name, self.bucket.id, created_at, count_str]
    }
}
impl Serialize for PrintableBucket {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.bucket, serializer)
    }
}

/// Source with additional fields for printing
/// Serializes to a Source
#[derive(Debug)]
pub struct PrintableSource {
    pub source: Source,
    pub bucket: Option<Bucket>,
    pub stats: Option<SourceStatistics>,
}

impl Serialize for PrintableSource {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.source, serializer)
    }
}

impl DisplayTable for PrintableSource {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Transform Tag", "Bucket", "Title", "Num Comments"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!(
            "{}{}{}",
            self.source.owner.dimmed(),
            "/".dimmed(),
            self.source.name
        );
        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.source.updated_at)
            .unwrap_or_else(|_| {
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
            })
            .format("%Y-%m-%d %H:%M:%S");
        row![
            full_name,
            self.source.id,
            updated_at,
            match &self.source.email_transform_tag {
                Some(transform_tag) => transform_tag.as_str().into(),
                None => "missing".dimmed(),
            },
            match &self.bucket {
                Some(bucket) => bucket.name.as_str().into(),
                None => match &self.source.bucket_id {
                    Some(bucket_id) => bucket_id.as_str().dimmed(),
                    None => "none".dimmed(),
                },
            },
            self.source.title,
            if let Some(stats) = &self.stats {
                stats.num_comments.to_string().as_str().into()
            } else {
                "none".dimmed()
            }
        ]
    }
}

impl DisplayTable for KeyedSyncState {
    fn to_table_headers() -> Row {
        row![bFg => "Type", "Mailbox Name", "Folder Path", "Status", "Synced Until", "Last Synced At"]
    }

    fn to_table_row(&self) -> Row {
        let synced_until_str = if let Some(synced_until) = &self.synced_until {
            chrono::DateTime::parse_from_rfc3339(synced_until)
                .unwrap_or_else(|_| {
                    chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
                })
                .to_rfc2822()
                .normal()
        } else {
            "N/A".dimmed()
        };

        row![
            self.mailbox_name,
            self.folder_path.join("/"),
            self.status,
            synced_until_str,
            self.last_synced_at
        ]
    }
}

impl DisplayTable for ListKeyedSyncStatesResponseKeyedSyncStatesInner {
    fn to_table_headers() -> Row {
        row![bFg => "Mailbox Name", "Folder Path", "Status", "Synced Until", "Last Synced At"]
    }

    fn to_table_row(&self) -> Row {
        let synced_until_str = if let Some(synced_until) = &self.synced_until {
            chrono::DateTime::parse_from_rfc3339(synced_until)
                .unwrap_or_else(|_| {
                    chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
                })
                .to_rfc2822()
                .normal()
        } else {
            "N/A".dimmed()
        };

        row![
            if self.folder_id.is_none() {
                "Mailbox"
            } else {
                "Folder"
            },
            self.mailbox_name,
            if let Some(folder_path) = &self.folder_path {
                folder_path.join("/").normal()
            } else {
                "N/A".dimmed()
            },
            self.status,
            synced_until_str,
            self.last_synced_at
        ]
    }
}

impl DisplayTable for Trigger {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Title"]
    }

    fn to_table_row(&self) -> Row {
        let updated_at = match &self.updated_at {
            Some(updated) => chrono::DateTime::parse_from_rfc3339(updated)
                .unwrap_or_else(|_| {
                    chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
                })
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            None => "N/A".dimmed().to_string(),
        };
        row![self.name, self.id, updated_at, self.title]
    }
}

impl DisplayTable for User {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "Email", "ID", "Created (UTC)", "Global Permissions"]
    }

    fn to_table_row(&self) -> Row {
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.created)
            .unwrap_or_else(|_| {
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
            })
            .format("%Y-%m-%d %H:%M:%S");
        row![
            self.username,
            self.email,
            self.id,
            created_at,
            self.global_permissions
                .iter()
                .chain(self.sso_global_permissions.iter())
                .map(|permission| permission.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        ]
    }
}

// Custom struct for printable audit events (OpenAPI equivalent)
#[derive(Debug)]
pub struct PrintableAuditEvent {
    pub event: AuditEvent,
    pub dataset_names: Vec<String>,
    pub project_names: Vec<String>,
    pub tenant_names: Vec<String>,
    pub actor_email: String,
    pub actor_tenant_name: String,
}

impl DisplayTable for PrintableAuditEvent {
    fn to_table_headers() -> Row {
        row![bFg => "Timestamp", "Event Id", "Event Type", "Actor Email", "Actor Tenant", "Dataset Names",  "Project Names", "Tenant Names"]
    }

    fn to_table_row(&self) -> Row {
        row![
            self.event.timestamp,
            self.event.event_id,
            self.event.event_type,
            self.actor_email,
            self.actor_tenant_name,
            if self.dataset_names.is_empty() {
                "none".dimmed()
            } else {
                self.dataset_names
                    .iter()
                    .map(|dataset| dataset.clone())
                    .collect::<Vec<String>>()
                    .join(" & ")
                    .normal()
            },
            if self.project_names.is_empty() {
                "none".dimmed()
            } else {
                self.project_names
                    .iter()
                    .map(|project| project.clone())
                    .collect::<Vec<String>>()
                    .join(" & ")
                    .normal()
            },
            if self.tenant_names.is_empty() {
                "none".dimmed()
            } else {
                self.tenant_names
                    .iter()
                    .map(|name| name.clone())
                    .collect::<Vec<String>>()
                    .join(" & ")
                    .normal()
            }
        ]
    }
}

impl Serialize for PrintableAuditEvent {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&self.event, serializer)
    }
}

/// Helper trait to allow collection of resources to be converted into a table.
pub trait IntoTable {
    fn into_table(self) -> Table;
}

/// All iterators of resources can be converted into a table.
impl<'a, Iterable, Item: 'a> IntoTable for Iterable
where
    Iterable: IntoIterator<Item = &'a Item>,
    Item: DisplayTable,
{
    fn into_table(self) -> Table {
        let mut table = new_table();
        table.set_titles(Item::to_table_headers());
        for source in self.into_iter() {
            table.add_row(source.to_table_row());
        }
        table
    }
}

fn new_table() -> Table {
    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .separators(&[], format::LineSeparator::new('-', '+', '+', '+'))
        .padding(0, 1)
        .build();
    table.set_format(format);
    table
}

fn print_table<T: IntoTable>(resources: T) {
    let table = resources.into_table();
    table.printstd();
}

/// Print resources using the selected output format.
///
/// Resources passed to the printer must be able to be formatted using all supported
/// `OutputFormat`s.
#[derive(Default, Debug)]
pub struct Printer {
    output: OutputFormat,
}

impl Printer {
    pub fn new(output: OutputFormat) -> Self {
        Self { output }
    }

    pub fn print_resources<T, Resource>(&self, resources: T) -> Result<()>
    where
        T: IntoIterator<Item = Resource> + IntoTable,
        Resource: Serialize,
    {
        match self.output {
            OutputFormat::Table => print_table(resources),
            OutputFormat::Json => print_resources_as_json(resources, io::stdout().lock())?,
        };
        Ok(())
    }
}
