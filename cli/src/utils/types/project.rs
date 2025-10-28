use anyhow::{anyhow, Result};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectName(pub String);

impl FromStr for ProjectName {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            Ok(ProjectName(s.to_owned()))
        } else {
            Err(anyhow!("project name must contain only alphanumeric characters, underscores, or hyphens, got '{s}'"))
        }
    }
}

impl ProjectName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
