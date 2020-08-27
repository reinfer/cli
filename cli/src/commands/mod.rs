#[cfg(feature = "alpha")]
pub mod alpha;
pub mod config;
pub mod create;
pub mod delete;
pub mod get;

use std::str::FromStr;

use crate::errors::{Error, ErrorKind, Result};

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
            Err(ErrorKind::UnknownOutputFormat(string.into()).into())
        }
    }
}
