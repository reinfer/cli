use colored::Colorize;
use prettytable::{cell, format, row, Row, Table};
use reinfer_client::{Bucket, Dataset, Project, Source, Trigger, User};
use serde::Serialize;

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

#[derive(Copy, Clone, Debug)]
pub enum OutputFormat {
    Json,
    Table,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Table
    }
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

impl DisplayTable for Bucket {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Created (UTC)", "Updated (UTC)", "Transform Tag"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!("{}{}{}", self.owner.0.dimmed(), "/".dimmed(), self.name.0);
        row![
            full_name,
            self.id.0,
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
            self.updated_at.format("%Y-%m-%d %H:%M:%S"),
            match &self.transform_tag {
                Some(transform_tag) => transform_tag.0.as_str().into(),
                None => "missing".dimmed(),
            }
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
            self.title
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
        row![bFg => "Name", "ID", "Updated (UTC)", "Kind", "Title"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!("{}{}{}", self.owner.0.dimmed(), "/".dimmed(), self.name.0);
        row![
            full_name,
            self.id.0,
            self.updated_at.format("%Y-%m-%d %H:%M:%S"),
            self.kind,
            self.title
        ]
    }
}

impl DisplayTable for Trigger {
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
        row![bFg => "Name", "Email", "ID", "Created (UTC)"]
    }

    fn to_table_row(&self) -> Row {
        row![
            self.username.0,
            self.email.0,
            self.id.0,
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
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
