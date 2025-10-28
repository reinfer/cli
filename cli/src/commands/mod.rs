use std::{
    fs::{create_dir, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

use anyhow::{anyhow, Context, Result};
use dialoguer::Confirm;
use url::Url;

use crate::utils::types::transform::DEFAULT_TRANSFORM_TAG;

pub mod auth;
pub mod config;
pub mod create;
pub mod delete;
pub mod get;
pub mod package;
pub mod parse;
pub mod update;

pub fn ensure_uip_user_consents_to_ai_unit_charge(base_url: &Url) -> Result<()> {
    if base_url
        .origin()
        .ascii_serialization()
        .to_lowercase()
        .ends_with("reinfer.dev")
    {
        return Ok(());
    }

    if Confirm::new()
        .with_prompt(
            r#"🚨⚠️ 👉 CAUTION 👈⚠️ 🚨

The operation you are about to perform will charge AI units.

Do you want to continue?"#,
        )
        .interact()?
    {
        Ok(())
    } else {
        Err(anyhow!("Billable operation aborted by user"))
    }
}

pub struct LocalAttachmentPath {
    index: usize,
    name: String,
    parent_dir: PathBuf,
}

const INVALID_FILENAME_CHARS: [char; 9] = ['/', '<', '>', ':', '"', '\\', '|', '?', '*'];

fn clean_file_name(mut name: String) -> String {
    for char in INVALID_FILENAME_CHARS {
        name = name.replace(char, "□");
    }

    name
}

impl LocalAttachmentPath {
    fn ensure_parent_dir_exists(&self) -> Result<()> {
        if !self.parent_dir.exists() {
            create_dir(&self.parent_dir)?;
        }
        Ok(())
    }

    fn name(&self) -> String {
        format!("{0}.{1}", self.index, clean_file_name(self.name.clone()))
    }

    fn path(&self) -> PathBuf {
        self.parent_dir.join(self.name())
    }

    fn exists(&self) -> bool {
        self.path().is_file()
    }

    pub fn write(&self, buf_to_write: Vec<u8>) -> Result<bool> {
        self.ensure_parent_dir_exists()?;

        if !self.exists() {
            let f = File::create(self.path()).context("Could not create attachment output file")?;

            let mut buf_writer = BufWriter::new(f);
            buf_writer.write_all(&buf_to_write)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::clean_file_name;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_clean_file_name() {
        let filename = "this is a file 2024:08:07?";
        let cleaned = clean_file_name(filename.to_string());

        assert_eq!("this is a file 2024□08□07□", cleaned)
    }
}
