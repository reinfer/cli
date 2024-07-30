use std::{
    fs::{create_dir, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

use anyhow::{anyhow, Context, Result};
use dialoguer::Confirm;
use once_cell::sync::Lazy;
use reinfer_client::TransformTag;
use url::Url;

pub mod config;
pub mod create;
pub mod delete;
pub mod get;
pub mod parse;
pub mod update;

pub fn ensure_uip_user_consents_to_ai_unit_charge(base_url: &Url) -> Result<()> {
    if base_url
        .origin()
        .ascii_serialization()
        .to_lowercase()
        .ends_with("reinfer.io")
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

static DEFAULT_TRANSFORM_TAG: Lazy<TransformTag> =
    Lazy::new(|| TransformTag("generic.0.CONVKER5".to_string()));

pub struct LocalAttachmentPath {
    index: usize,
    name: String,
    parent_dir: PathBuf,
}

impl LocalAttachmentPath {
    fn ensure_parent_dir_exists(&self) -> Result<()> {
        if !self.parent_dir.exists() {
            create_dir(&self.parent_dir)?;
        }
        Ok(())
    }

    fn name(&self) -> String {
        format!("{0}.{1}", self.index, self.name)
    }

    fn path(&self) -> PathBuf {
        self.parent_dir.join(self.name())
    }

    pub fn write(&self, buf_to_write: Vec<u8>) -> Result<bool> {
        self.ensure_parent_dir_exists()?;

        if !self.path().is_file() {
            let f = File::create(self.path()).context("Could not create attachment output file")?;

            let mut buf_writer = BufWriter::new(f);
            buf_writer.write_all(&buf_to_write)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
