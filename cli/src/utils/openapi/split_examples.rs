//! High-level wrapper functions for OpenAPI split-on-failure operations
//!
//! This module provides clean, easy-to-use wrapper functions that add automatic
//! resilience to OpenAPI batch operations. When batch requests fail with certain
//! error conditions, these functions automatically split the request into individual
//! operations and retry them, allowing for partial success instead of complete failure.

use anyhow::{Context, Result};
use openapi::{
    apis::{
        comments_api::{add_comments, sync_comments},
        configuration::Configuration,
        emails_api::add_emails_to_bucket,
    },
    models::{
        AddCommentsRequest, AddCommentsResponse, AddEmailsToBucketRequest,
        AddEmailsToBucketResponse, CommentNew, EmailNew, SyncCommentsRequest, SyncCommentsResponse,
    },
};

use super::split_failure::{execute_with_split_on_failure, SplitOnFailureResult};

/// Add comments with automatic split-on-failure resilience
///
/// This function wraps the `add_comments` OpenAPI call with automatic retry logic.
/// If the batch request fails with certain error conditions (422, 400, timeouts),
/// it automatically splits the request into individual comment operations and retries them.
pub fn add_comments_with_split_on_failure(
    config: &Configuration,
    owner: &str,
    source_name: &str,
    comments: Vec<CommentNew>,
    no_charge: Option<bool>,
) -> Result<SplitOnFailureResult<AddCommentsResponse>> {
    let request = AddCommentsRequest::new(comments);

    // Wrap the API call with parameters
    let api_call =
        |req: AddCommentsRequest| add_comments(config, owner, source_name, req, no_charge);

    execute_with_split_on_failure(api_call, request, "add_comments")
        .context("Failed to add comments with split-on-failure")
}

/// Sync comments with automatic split-on-failure resilience  
///
/// This function wraps the `sync_comments` OpenAPI call with automatic retry logic.
/// Similar to `add_comments_with_split_on_failure` but for sync operations which return
/// detailed statistics that get properly merged across split requests.
pub fn sync_comments_with_split_on_failure(
    config: &Configuration,
    owner: &str,
    source_name: &str,
    comments: Vec<CommentNew>,
    no_charge: Option<bool>,
) -> Result<SplitOnFailureResult<SyncCommentsResponse>> {
    let request = SyncCommentsRequest::new(comments);

    let api_call =
        |req: SyncCommentsRequest| sync_comments(config, owner, source_name, req, no_charge);

    execute_with_split_on_failure(api_call, request, "sync_comments")
        .context("Failed to sync comments with split-on-failure")
}

/// Add emails to bucket with automatic split-on-failure resilience
///
/// This function wraps the `add_emails_to_bucket` OpenAPI call with automatic retry logic.
/// If the batch request fails, it automatically splits the request into individual email operations.
pub fn add_emails_to_bucket_with_split_on_failure(
    config: &Configuration,
    owner: &str,
    bucket_name: &str,
    emails: Vec<EmailNew>,
    no_charge: Option<bool>,
) -> Result<SplitOnFailureResult<AddEmailsToBucketResponse>> {
    let request = AddEmailsToBucketRequest::new(emails);

    let api_call = |req: AddEmailsToBucketRequest| {
        add_emails_to_bucket(config, owner, bucket_name, req, no_charge)
    };

    execute_with_split_on_failure(api_call, request, "add_emails_to_bucket")
        .context("Failed to add emails to bucket with split-on-failure")
}

/// Display results of split-on-failure operations with user-friendly messages
///
/// This helper function provides consistent status reporting for split-on-failure operations,
/// displaying success, partial failure, or complete failure with appropriate icons and details.
pub fn handle_split_on_failure_result<T>(
    result: SplitOnFailureResult<T>,
    total_items: usize,
    operation_name: &str,
) {
    let successful = total_items - result.num_failed;

    if result.num_failed == 0 {
        println!(
            "✅ {} completed successfully: {}/{} items processed",
            operation_name, successful, total_items
        );
    } else if successful > 0 {
        println!(
            "⚠️  {} partially completed: {}/{} items processed, {} failed",
            operation_name, successful, total_items, result.num_failed
        );
    } else {
        println!(
            "❌ {} failed completely: 0/{} items processed",
            operation_name, total_items
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_result_display() {
        // Test complete success
        let success_result = SplitOnFailureResult {
            response: AddCommentsResponse::default(),
            num_failed: 0,
        };
        handle_split_on_failure_result(success_result, 10, "test_operation");

        // Test partial success
        let partial_result = SplitOnFailureResult {
            response: AddCommentsResponse::default(),
            num_failed: 3,
        };
        handle_split_on_failure_result(partial_result, 10, "test_operation");

        // Test complete failure
        let failure_result = SplitOnFailureResult {
            response: AddCommentsResponse::default(),
            num_failed: 10,
        };
        handle_split_on_failure_result(failure_result, 10, "test_operation");
    }
}
