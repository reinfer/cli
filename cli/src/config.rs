use anyhow::{Context, Result};
use log::debug;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ReinferConfig {
    current_context: Option<String>,
    contexts: Vec<ContextConfig>,
    #[serde(default)]
    pub context_is_required: bool,
}

impl ReinferConfig {
    pub fn get_all_contexts(&self) -> &Vec<ContextConfig> {
        &self.contexts
    }

    pub fn get_context(&self, name: &str) -> Option<&ContextConfig> {
        self.contexts.iter().find(|context| context.name == name)
    }

    pub fn set_context(&mut self, context: ContextConfig) -> bool {
        if let Some(index) = self.context_position(&context.name) {
            self.contexts[index] = context;
            true
        } else {
            self.contexts.push(context);
            false
        }
    }
    pub fn delete_context(&mut self, name: &str) -> bool {
        if let Some(index) = self.context_position(name) {
            self.contexts.remove(index);
            if self
                .current_context
                .as_ref()
                .map_or(false, |current_context| current_context == name)
            {
                self.current_context = None
            }
            true
        } else {
            false
        }
    }

    pub fn get_current_context(&self) -> Option<&ContextConfig> {
        self.current_context
            .as_ref()
            .and_then(|current_context| self.get_context(current_context))
    }

    pub fn set_current_context(&mut self, name: &str) -> bool {
        if self.get_context(name).is_some() {
            self.current_context = Some(name.to_owned());
            true
        } else {
            false
        }
    }

    pub fn num_contexts(&self) -> usize {
        self.contexts.len()
    }

    fn context_position(&self, name: &str) -> Option<usize> {
        self.contexts
            .iter()
            .position(|context| context.name == name)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextConfig {
    pub name: String,
    pub endpoint: Url,
    pub token: Option<String>,
    pub accept_invalid_certificates: bool,
    pub proxy: Option<Url>,
}

pub fn read_reinfer_config(path: impl AsRef<Path>) -> Result<ReinferConfig> {
    debug!("Reading config file at `{}`", path.as_ref().display());
    if path.as_ref().exists() {
        let file = File::open(&path)
            .with_context(|| format!("Could not open config file `{}`", path.as_ref().display()))?;
        let config_reader = BufReader::new(file);
        serde_json::from_reader(config_reader)
            .with_context(|| format!("Could not parse config file `{}`", path.as_ref().display()))
    } else {
        Ok(Default::default())
    }
}

pub fn write_reinfer_config(path: impl AsRef<Path>, config: &ReinferConfig) -> Result<()> {
    debug!("Writing config file at `{}`", path.as_ref().display());
    let file = File::create(&path)
        .with_context(|| format!("Could not create config file `{}`", path.as_ref().display()))?;
    let config_writer = BufWriter::new(file);
    serde_json::to_writer_pretty(config_writer, &config).with_context(|| {
        format!(
            "Could not serialise configuration to `{}`",
            path.as_ref().display()
        )
    })
}
