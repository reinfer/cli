pub mod api_utils;
pub mod attachment_utils;
pub mod attribute_filter_enum;
pub mod audit_events_iterator;
pub mod comment_timestamp_filter;
pub mod comments_iter_timerange;
pub mod comment_utils;
pub mod dataset_query_iter;
pub mod get_current_user;
pub mod id;
pub mod full_name;
pub mod label_name;
pub mod model_version;
pub mod new_annotated_comment;
pub mod openapi_split_on_failure;
pub mod openapi_split_examples;
pub mod permissions;
pub mod project_name;
pub mod refresh_permissions;
pub mod resource_identifier;
pub mod retry;
pub mod stream_full_name;
pub mod transform_tag;
pub mod user_properties_filter;
pub mod query_comments_csv;
pub mod utils;

// Re-export the type aliases for convenience
pub use api_utils::{upload_ixp_document_bytes, get_document_bytes, set_comment_audio};
pub use attachment_utils::AttachmentExt;
pub use audit_events_iterator::AuditEventsResponseExt;
pub use comment_timestamp_filter::CommentTimestampFilter;
pub use comments_iter_timerange::CommentsIterTimerange;
pub use dataset_query_iter::get_dataset_query_iter;
pub use full_name::FullName;
pub use get_current_user::get_current_user;
pub use id::CommentId;
pub use label_name::LabelName;
pub use model_version::ModelVersion;
pub use new_annotated_comment::NewAnnotatedComment;
pub use openapi_split_examples::{
    add_comments_with_split_on_failure, sync_comments_with_split_on_failure,
    add_emails_to_bucket_with_split_on_failure, handle_split_on_failure_result
};
pub use permissions::{GlobalPermission, ProjectPermission};
pub use project_name::ProjectName;
pub use resource_identifier::{BucketIdentifier, DatasetIdentifier, SourceIdentifier};
pub use id::SourceId;

pub use stream_full_name::StreamFullName;
pub use user_properties_filter::UserPropertiesFilter;

// Type aliases for backward compatibility
pub type DatasetFullName = FullName;

// Constants missing from OpenAPI - simplified version
pub const DEFAULT_LABEL_GROUP_NAME: &str = "";
