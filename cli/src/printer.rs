use super::thousands::Thousands;
use colored::Colorize;
use prettytable::{format, row, Row, Table};
use reinfer_client::{
    resources::{
        audit::PrintableAuditEvent, bucket_statistics::Statistics as BucketStatistics,
        dataset::DatasetAndStats, integration::Integration, quota::Quota,
    },
    Bucket, CommentStatistics, Dataset, Project, Source, Stream, User,
};
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
        row![
            self.owner.0,
            self.name.0,
            self.id.0,
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
            self.configuration.mailboxes.len()
        ]
    }
}
impl DisplayTable for Bucket {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Created (UTC)", "Transform Tag"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!("{}{}{}", self.owner.0.dimmed(), "/".dimmed(), self.name.0);
        row![
            full_name,
            self.id.0,
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
            match &self.transform_tag {
                Some(transform_tag) => transform_tag.0.as_str().into(),
                None => "missing".dimmed(),
            }
        ]
    }
}

impl DisplayTable for Quota {
    fn to_table_headers() -> Row {
        row![bFg => "Kind", "Hard Limit", "Usage (Total)", "Usage %"]
    }

    fn to_table_row(&self) -> Row {
        row![
            self.quota_kind,
            Thousands(self.hard_limit),
            Thousands(self.current_max_usage),
            format!(
                "{:.0}%",
                (self.current_max_usage as f64 / self.hard_limit as f64) * 100.0
            )
        ]
    }
}

impl DisplayTable for Dataset {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Title"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!("{}{}{}", self.owner.0.dimmed(), "/".dimmed(), self.name.0);
        row![
            full_name,
            self.id.0,
            self.updated_at.format("%Y-%m-%d %H:%M:%S"),
            self.title,
        ]
    }
}

impl DisplayTable for DatasetAndStats {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Title","Total Verbatims", "Num Reviewed"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!(
            "{}{}{}",
            self.dataset.owner.0.dimmed(),
            "/".dimmed(),
            self.dataset.name.0
        );
        row![
            full_name,
            self.dataset.id.0,
            self.dataset.updated_at.format("%Y-%m-%d %H:%M:%S"),
            self.dataset.title,
            self.stats.total_verbatims,
            self.stats.num_reviewed
        ]
    }
}

impl DisplayTable for Project {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Title"]
    }

    fn to_table_row(&self) -> Row {
        row![
            self.name.0,
            match &self.id {
                Some(id) => id.0.as_str().into(),
                None => "unknown".dimmed(),
            },
            self.title
        ]
    }
}

impl DisplayTable for Source {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Transform Tag", "Title"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!("{}{}{}", self.owner.0.dimmed(), "/".dimmed(), self.name.0);
        row![
            full_name,
            self.id.0,
            self.updated_at.format("%Y-%m-%d %H:%M:%S"),
            match &self.transform_tag {
                Some(transform_tag) => transform_tag.0.as_str().into(),
                None => "missing".dimmed(),
            },
            self.title,
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
        row![bFg => "Name", "ID", "Created (UTC)", "Transform Tag", "Num Emails"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!(
            "{}{}{}",
            self.bucket.owner.0.dimmed(),
            "/".dimmed(),
            self.bucket.name.0
        );
        row![
            full_name,
            self.bucket.id.0,
            self.bucket.created_at.format("%Y-%m-%d %H:%M:%S"),
            match &self.bucket.transform_tag {
                Some(transform_tag) => transform_tag.0.as_str().into(),
                None => "missing".dimmed(),
            },
            if let Some(stats) = &self.stats {
                stats.count.to_string().as_str().into()
            } else {
                "none".dimmed()
            }
        ]
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
    pub stats: Option<CommentStatistics>,
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
            self.source.owner.0.dimmed(),
            "/".dimmed(),
            self.source.name.0
        );
        row![
            full_name,
            self.source.id.0,
            self.source.updated_at.format("%Y-%m-%d %H:%M:%S"),
            match &self.source.transform_tag {
                Some(transform_tag) => transform_tag.0.as_str().into(),
                None => "missing".dimmed(),
            },
            match &self.bucket {
                Some(bucket) => bucket.name.0.as_str().into(),
                None => match &self.source.bucket_id {
                    Some(bucket_id) => bucket_id.0.as_str().dimmed(),
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

impl DisplayTable for Stream {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Title"]
    }

    fn to_table_row(&self) -> Row {
        row![
            self.name.0,
            self.id.0,
            self.updated_at.format("%Y-%m-%d %H:%M:%S"),
            self.title
        ]
    }
}

impl DisplayTable for User {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "Email", "ID", "Created (UTC)", "Global Permissions"]
    }

    fn to_table_row(&self) -> Row {
        row![
            self.username.0,
            self.email.0,
            self.id.0,
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
            self.global_permissions
                .iter()
                .chain(self.sso_global_permissions.iter())
                .map(|permission| permission.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        ]
    }
}

impl DisplayTable for PrintableAuditEvent {
    fn to_table_headers() -> Row {
        row![bFg => "Timestamp", "Event Id", "Event Type", "Actor Email", "Actor Tenant", "Dataset Names",  "Project Names", "Tenant Names"]
    }

    fn to_table_row(&self) -> Row {
        row![
            self.timestamp,
            self.event_id.0,
            self.event_type.0,
            self.actor_email.0,
            self.actor_tenant_name.0,
            if self.dataset_names.is_empty() {
                "none".dimmed()
            } else {
                self.dataset_names
                    .iter()
                    .map(|dataset| dataset.0.clone())
                    .collect::<Vec<String>>()
                    .join(" & ")
                    .normal()
            },
            if self.project_names.is_empty() {
                "none".dimmed()
            } else {
                self.project_names
                    .iter()
                    .map(|project| project.0.clone())
                    .collect::<Vec<String>>()
                    .join(" & ")
                    .normal()
            },
            if self.tenant_names.is_empty() {
                "none".dimmed()
            } else {
                self.tenant_names
                    .iter()
                    .map(|name| name.0.clone())
                    .collect::<Vec<String>>()
                    .join(" & ")
                    .normal()
            }
        ]
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
