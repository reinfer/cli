//! Refresh user permissions functionality using OpenAPI Configuration
//!
//! This module provides the same functionality as client.refresh_user_permissions()
//! but using the OpenAPI Configuration instead of the legacy Client.

use anyhow::{Context, Result};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::utils::retry::retry_request;
use openapi::apis::configuration::Configuration;

/// Request structure for refreshing user permissions
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshUserPermissionsRequest {}

/// Response structure for refreshing user permissions
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshUserPermissionsResponse {
    pub permissions_refreshed: Option<bool>,
}

/// Refresh user permissions using OpenAPI Configuration
///
/// This function replicates the behavior of client.refresh_user_permissions()
/// but uses the OpenAPI Configuration instead of the legacy Client.
pub fn refresh_user_permissions(config: &Configuration) -> Result<RefreshUserPermissionsResponse> {
    // Construct the endpoint URL
    let base_url = &config.base_path;
    let endpoint = format!(
        "{}/auth/refresh-user-permissions",
        base_url.trim_end_matches('/')
    );

    // Create the request body (empty struct)
    let request_body = RefreshUserPermissionsRequest {};

    // Build the HTTP request
    let mut request = config
        .client
        .request(Method::POST, &endpoint)
        .json(&request_body);

    // Add authorization header if we have a bearer token
    if let Some(ref token) = config.bearer_access_token {
        request = request.bearer_auth(token);
    }

    // Add any additional headers from the config
    if let Some(ref user_agent) = config.user_agent {
        request = request.header(reqwest::header::USER_AGENT, user_agent);
    }

    // Send the request with retry logic (similar to the client's Retry::Yes behavior)
    let response = retry_request(|| {
        Ok(request
            .try_clone()
            .context("Failed to clone request for retry")?
            .send()?)
    })?;

    // Check if the response is successful
    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow::anyhow!(
            "Refresh user permissions failed with status {}: {}",
            status,
            error_text
        ));
    }

    // Parse the response
    let refresh_response: RefreshUserPermissionsResponse = response
        .json()
        .context("Failed to parse refresh user permissions response")?;

    Ok(refresh_response)
}
