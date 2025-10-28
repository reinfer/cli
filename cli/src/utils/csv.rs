/*
 * CSV Query Comments Utility
 *
 * This module provides a CSV version of the OpenAPI-generated `query_comments` function.
 *
 * ## Background
 * The OpenAPI-generated client in `client/src/apis/comments_api.rs` provides a `query_comments`
 * function that returns structured JSON data as `QueryCommentsResponse`. However, sometimes you
 * need the same data exported as CSV for spreadsheets, reports, or file processing.
 *
 * ## What this does
 * - Hits the SAME endpoint as `query_comments`: `/api/_private/datasets/{owner}/{dataset_name}/query`
 * - Uses the SAME parameters: owner, dataset_name, QueryCommentsRequest, limit, continuation, etc.
 * - But requests CSV format by setting Accept header to "text/csv"
 * - Returns raw CSV text as String instead of parsed JSON objects
 *
 * ## Usage Example
 * ```rust
 * use openapi::models::QueryCommentsRequest;
 *
 * let csv_data = query_comments_csv(
 *     &config,
 *     "my-owner",
 *     "my-dataset",
 *     QueryCommentsRequest::new(/* ... */),
 *     Some(1000),    // limit
 *     None,          // continuation
 *     None,          // collapse_mode
 *     None,          // order
 * )?;
 *
 * // csv_data is now a String like:
 * // "id,timestamp,message,labels\n123,2023-01-01,Hello,positive\n456,2023-01-02,Goodbye,neutral\n"
 * ```
 *
 * ## Relationship to manual client
 * This mirrors the pattern used in the manual client (`api/src/lib.rs`) where there are
 * both `query_dataset()` and `query_dataset_csv()` functions - same endpoint, different response formats.
 */

use anyhow::Result;
use openapi::apis::configuration::Configuration;
use openapi::models;

/// CSV version of query_comments - returns raw CSV text instead of structured JSON
///
/// This function is identical to the OpenAPI-generated `query_comments` except:
/// 1. Sets Accept header to "text/csv" to request CSV format
/// 2. Returns String containing CSV data instead of QueryCommentsResponse struct
/// 3. Does not attempt to JSON-parse the response
#[allow(clippy::too_many_arguments)]
pub fn query_comments_csv(
    configuration: &Configuration,
    owner: &str,
    dataset_name: &str,
    query_comments_request: models::QueryCommentsRequest,
    limit: Option<i32>,
    continuation: Option<&str>,
    collapse_mode: Option<&str>,
    order: Option<&str>,
) -> Result<String> {
    let local_var_configuration = configuration;
    let local_var_client = &local_var_configuration.client;

    // Same endpoint as query_comments
    let local_var_uri_str = format!(
        "{}/api/_private/datasets/{owner}/{dataset_name}/query",
        local_var_configuration.base_path,
        owner = openapi::apis::urlencode(owner),
        dataset_name = openapi::apis::urlencode(dataset_name)
    );

    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    // Add all the same query parameters as original function
    if let Some(ref local_var_str) = limit {
        local_var_req_builder =
            local_var_req_builder.query(&[("limit", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = continuation {
        local_var_req_builder =
            local_var_req_builder.query(&[("continuation", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = collapse_mode {
        local_var_req_builder =
            local_var_req_builder.query(&[("collapse_mode", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = order {
        local_var_req_builder =
            local_var_req_builder.query(&[("order", &local_var_str.to_string())]);
    }

    // Add authentication headers (same as original)
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{local_var_prefix} {local_var_key}"),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("authorization", local_var_value);
    }

    // KEY DIFFERENCE: Set Accept header to request CSV format instead of JSON
    local_var_req_builder = local_var_req_builder.header("accept", "text/csv");

    // Send the same request body as original function
    local_var_req_builder = local_var_req_builder.json(&query_comments_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req)?;

    let local_var_status = local_var_resp.status();

    if local_var_status.is_success() {
        // SUCCESS: Return raw CSV text as String (not parsed as JSON)
        let csv_content = local_var_resp.text()?;
        Ok(csv_content)
    } else {
        // ERROR: Still try to parse error as JSON for better error messages
        let local_var_content = local_var_resp.text()?;
        let error_msg = format!(
            "CSV query failed with status {local_var_status}: {local_var_content}"
        );
        Err(anyhow::anyhow!(error_msg))
    }
}
