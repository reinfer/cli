use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshUserPermissionsRequest {}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshUserPermissionsResponse {
    pub permissions_refreshed: Option<bool>,
}
