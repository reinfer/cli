use crate::{Error, Result};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReinferTenantId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiPathTenantId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TenantId {
    Reinfer(ReinferTenantId),
    UiPath(UiPathTenantId),
}

impl Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TenantId::Reinfer(ReinferTenantId(tenant_id))
                | TenantId::UiPath(UiPathTenantId(tenant_id)) => tenant_id,
            }
        )
    }
}

impl FromStr for ReinferTenantId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self(s.to_string()))
    }
}

impl FromStr for UiPathTenantId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self(s.to_string()))
    }
}
