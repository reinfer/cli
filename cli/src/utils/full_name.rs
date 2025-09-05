use anyhow::{anyhow, Result};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullName(pub String);

impl FromStr for FullName {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.split('/').count() == 2 {
            Ok(FullName(s.to_owned()))
        } else {
            Err(anyhow!("expected owner/name, got '{s}'"))
        }
    }
}

impl FullName {
    pub fn owner(&self) -> &str { self.0.split('/').next().unwrap() }
    pub fn name(&self)  -> &str { self.0.split('/').nth(1).unwrap() }
    pub fn as_str(&self) -> &str { &self.0 }
}
