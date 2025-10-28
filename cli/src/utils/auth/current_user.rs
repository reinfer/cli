use anyhow::{Context, Result};
use openapi::{apis::configuration::Configuration, models::User};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::utils::retry::retry_request;

// Response model for the /auth/user endpoint (not available in OpenAPI spec)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetCurrentUserResponse {
    pub user: User,
}

/// Get the current user associated with the API token in use.
///
/// This function makes a direct HTTP request to the /auth/user endpoint
/// since this endpoint is not available in the OpenAPI specification.
///
/// # Arguments
/// * `config` - The OpenAPI configuration containing authentication and client info
///
/// # Returns
/// * `Result<User>` - The current user on success, or an error on failure
pub fn get_current_user(config: &Configuration) -> Result<User> {
    // Construct the endpoint URL (this endpoint is not available in OpenAPI spec)
    let base_url = &config.base_path;
    let endpoint = format!("{}/auth/user", base_url.trim_end_matches('/'));

    // Build the HTTP request
    let mut request = config.client.request(Method::GET, &endpoint);

    // Add authorization header if we have a bearer token
    if let Some(ref token) = config.bearer_access_token {
        request = request.bearer_auth(token);
    }

    // Add any additional headers from the config
    if let Some(ref user_agent) = config.user_agent {
        request = request.header(reqwest::header::USER_AGENT, user_agent);
    }

    // Send the request with retry logic (similar to refresh_permissions)
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
            "Operation to get the current user has failed with status {}: {}",
            status,
            error_text
        ));
    }

    // Parse the response
    let current_user_response: GetCurrentUserResponse = response
        .json()
        .context("Failed to parse get current user response")?;

    Ok(current_user_response.user)
}
