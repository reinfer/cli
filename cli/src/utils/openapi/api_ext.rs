use anyhow::Result;
use openapi::{
    apis::{
        comments_api::{GetCommentAudioError, SetCommentAudioError},
        configuration::Configuration,
        ixp_models_api::UploadIxpDocumentError,
        Error, ResponseContent,
    },
    models::{IxpUploadDocumentResponse, SetCommentAudioResponse},
};
use reqwest;
use std::path::Path;

/// Upload IXP Document for Predictions (using bytes)
pub fn upload_ixp_document_bytes(
    configuration: &Configuration,
    project_uuid: &str,
    model_version: &str,
    bytes: Option<Vec<u8>>,
    filename: Option<String>,
) -> Result<IxpUploadDocumentResponse, Error<UploadIxpDocumentError>> {
    let local_var_configuration = configuration;
    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/_private/ixp/projects/{project_uuid}/models/{model_version}/documents",
        local_var_configuration.base_path,
        project_uuid = openapi::apis::urlencode(project_uuid),
        model_version = openapi::apis::urlencode(model_version)
    );

    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::PUT, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("authorization", local_var_value);
    };

    let mut local_var_form = reqwest::blocking::multipart::Form::new();
    if let Some(local_var_param_value) = bytes {
        let mut part = reqwest::blocking::multipart::Part::bytes(local_var_param_value);
        if let Some(file_name) = filename {
            part = part.file_name(file_name);
        }
        local_var_form = local_var_form.part("file", part);
    }
    local_var_req_builder = local_var_req_builder.multipart(local_var_form);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req)?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text()?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<UploadIxpDocumentError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get document as bytes (fixing the generated get_document that incorrectly tries to parse binary as JSON)
pub fn get_document_bytes(
    configuration: &Configuration,
    source_id: &str,
    comment_id: &str,
) -> Result<Vec<u8>, Error<openapi::apis::documents_api::GetDocumentError>> {
    let local_var_configuration = configuration;
    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/_private/sources/id:{source_id}/documents/{comment_id}",
        local_var_configuration.base_path,
        source_id = openapi::apis::urlencode(source_id),
        comment_id = openapi::apis::urlencode(comment_id)
    );

    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("authorization", local_var_value);
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req)?;

    let local_var_status = local_var_resp.status();

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        // Get bytes instead of trying to parse as JSON
        let bytes = local_var_resp.bytes()?;
        Ok(bytes.to_vec())
    } else {
        let local_var_content = local_var_resp.text()?;
        let local_var_entity: Option<openapi::apis::documents_api::GetDocumentError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get the audio for a comment (returns raw bytes, not parsed as JSON)
/// This is a fixed version of the broken generated get_comment_audio function.
/// Currently unused but available for when needed.
#[allow(dead_code)]
pub fn get_comment_audio(
    configuration: &Configuration,
    source_id: &str,
    comment_id: &str,
) -> Result<Vec<u8>, Error<GetCommentAudioError>> {
    let local_var_configuration = configuration;
    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/_private/sources/id:{source_id}/comments/{comment_id}/audio",
        local_var_configuration.base_path,
        source_id = openapi::apis::urlencode(source_id),
        comment_id = openapi::apis::urlencode(comment_id)
    );

    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("authorization", local_var_value);
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req)?;

    let local_var_status = local_var_resp.status();

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        // Return raw bytes, not JSON parsed content
        let bytes = local_var_resp.bytes()?;
        Ok(bytes.to_vec())
    } else {
        let local_var_content = local_var_resp.text()?;
        let local_var_entity: Option<GetCommentAudioError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Set the audio for a comment (uses proper multipart file upload, unlike the broken generated version)
pub fn set_comment_audio(
    configuration: &Configuration,
    raw_source_id: &str,
    raw_comment_id: &str,
    audio_path: impl AsRef<Path>,
) -> Result<SetCommentAudioResponse, Error<SetCommentAudioError>> {
    let local_var_configuration = configuration;
    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/_private/sources/id:{raw_source_id}/comments/{raw_comment_id}/audio",
        local_var_configuration.base_path,
        raw_source_id = openapi::apis::urlencode(raw_source_id),
        raw_comment_id = openapi::apis::urlencode(raw_comment_id)
    );

    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::PUT, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("authorization", local_var_value);
    };

    // Use proper multipart form (like the working legacy version), NOT JSON like the broken generated version
    let local_var_form = reqwest::blocking::multipart::Form::new()
        .file("file", audio_path)
        .map_err(|source| {
            Error::ResponseError(ResponseContent {
                status: reqwest::StatusCode::BAD_REQUEST,
                content: format!("Failed to create multipart form for file: {}", source),
                entity: None,
            })
        })?;

    local_var_req_builder = local_var_req_builder.multipart(local_var_form);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req)?;

    let local_var_status = local_var_resp.status();

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        let local_var_content = local_var_resp.text()?;
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_content = local_var_resp.text()?;
        let local_var_entity: Option<SetCommentAudioError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}
