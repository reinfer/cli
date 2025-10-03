use anyhow::Result;
use openapi::{
    apis::{configuration::Configuration, ResponseContent, Error, ixp_models_api::UploadIxpDocumentError},
    models::IxpUploadDocumentResponse,
};
use reqwest;

/// Upload IXP Document for Predictions (using bytes)
pub fn upload_ixp_document_bytes(
    configuration: &Configuration, 
    project_uuid: &str, 
    model_version: &str, 
    bytes: Option<Vec<u8>>,
    filename: Option<String>
) -> Result<IxpUploadDocumentResponse, Error<UploadIxpDocumentError>> {
    let local_var_configuration = configuration;
    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/_private/ixp/projects/{project_uuid}/models/{model_version}/documents", 
        local_var_configuration.base_path, 
        project_uuid = openapi::apis::urlencode(project_uuid), 
        model_version = openapi::apis::urlencode(model_version)
    );
    
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::PUT, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
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
        let local_var_entity: Option<UploadIxpDocumentError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { 
            status: local_var_status, 
            content: local_var_content, 
            entity: local_var_entity 
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get document as bytes (fixing the generated get_document that incorrectly tries to parse binary as JSON)
pub fn get_document_bytes(
    configuration: &Configuration,
    source_id: &str,
    comment_id: &str
) -> Result<Vec<u8>, Error<openapi::apis::documents_api::GetDocumentError>> {
    let local_var_configuration = configuration;
    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/_private/sources/id:{source_id}/documents/{comment_id}",
        local_var_configuration.base_path,
        source_id = openapi::apis::urlencode(source_id),
        comment_id = openapi::apis::urlencode(comment_id)
    );
    
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
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
        let local_var_entity: Option<openapi::apis::documents_api::GetDocumentError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { 
            status: local_var_status, 
            content: local_var_content, 
            entity: local_var_entity 
        };
        Err(Error::ResponseError(local_var_error))
    }
}
