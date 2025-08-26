//! Utility modules for the CLI
//!
//! This module contains various utilities organized by functionality:
//! - `auth`: Authentication and user management utilities
//! - `types`: Type wrappers and identifiers for type safety
//! - `openapi`: OpenAPI client utilities and extensions
//! - `filters`: Query filtering and time range utilities
//! - `iterators`: Iterator utilities for paginated APIs
//! - `conversions`: Type conversion utilities between OpenAPI types
//! - `csv`: CSV export utilities
//! - `io`: I/O and logging utilities
//! - `retry`: Request retry utilities

// Organized modules
pub mod auth;
pub mod conversions;
pub mod csv;
pub mod filters;
pub mod io;
pub mod iterators;
pub mod openapi;
pub mod retry;
pub mod types;

// Standalone modules
pub mod attachment_utils;
pub mod new_annotated_comment;

// Re-export commonly used items for backward compatibility and convenience

// Auth utilities
pub use auth::{get_current_user, refresh_user_permissions, GlobalPermission, ProjectPermission};

// Type utilities
pub use types::{
    resolve_bucket, resolve_dataset, resolve_source, BucketIdentifier, CommentId, DatasetFullName,
    DatasetIdentifier, FullName, ModelVersion, ProjectName, SourceId, SourceIdentifier,
    StreamFullName,
};

// OpenAPI utilities
pub use openapi::{
    add_comments_with_split_on_failure, add_emails_to_bucket_with_split_on_failure,
    get_document_bytes, handle_split_on_failure_result, set_comment_audio,
    sync_comments_with_split_on_failure, upload_ixp_document_bytes,
};

// Filter utilities
pub use filters::{CommentTimestampFilter, CommentsIterTimerange, UserPropertiesFilter};

// Iterator utilities
pub use iterators::{get_dataset_query_iter, AuditEventsResponseExt};

// Conversion utilities are accessed via fully qualified paths (crate::utils::conversions::*)
// so they don't need to be re-exported here

// Other utilities
pub use attachment_utils::AttachmentExt;
pub use io::{read_from_stdin, read_token_from_stdin};
pub use new_annotated_comment::NewAnnotatedComment;

// Note: These are accessed via fully qualified paths:
// - csv::query_comments_csv (used as crate::utils::csv::query_comments_csv)
// - io::init_env_logger (used as crate::utils::io::init_env_logger)
// - retry::retry_request (used as crate::utils::retry::retry_request)

// Constants missing from OpenAPI - simplified version
pub const DEFAULT_LABEL_GROUP_NAME: &str = "";
