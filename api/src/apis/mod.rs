use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ResponseContent<T> {
    pub status: reqwest::StatusCode,
    pub content: String,
    pub entity: Option<T>,
}

#[derive(Debug)]
pub enum Error<T> {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
    ResponseError(ResponseContent<T>),
}

impl <T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (module, e) = match self {
            Error::Reqwest(e) => ("reqwest", e.to_string()),
            Error::Serde(e) => ("serde", e.to_string()),
            Error::Io(e) => ("IO", e.to_string()),
            Error::ResponseError(e) => ("response", format!("status code {}", e.status)),
        };
        write!(f, "error in {module}: {e}")
    }
}

impl <T: fmt::Debug> error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match self {
            Error::Reqwest(e) => e,
            Error::Serde(e) => e,
            Error::Io(e) => e,
            Error::ResponseError(_) => return None,
        })
    }
}

impl <T> From<reqwest::Error> for Error<T> {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl <T> From<serde_json::Error> for Error<T> {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

impl <T> From<std::io::Error> for Error<T> {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub fn urlencode<T: AsRef<str>>(s: T) -> String {
    ::url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

pub fn parse_deep_object(prefix: &str, value: &serde_json::Value) -> Vec<(String, String)> {
    if let serde_json::Value::Object(object) = value {
        let mut params = vec![];

        for (key, value) in object {
            match value {
                serde_json::Value::Object(_) => params.append(&mut parse_deep_object(
                    &format!("{prefix}[{key}]"),
                    value,
                )),
                serde_json::Value::Array(array) => {
                    for (i, value) in array.iter().enumerate() {
                        params.append(&mut parse_deep_object(
                            &format!("{prefix}[{key}][{i}]"),
                            value,
                        ));
                    }
                },
                serde_json::Value::String(s) => params.push((format!("{prefix}[{key}]"), s.clone())),
                _ => params.push((format!("{prefix}[{key}]"), value.to_string())),
            }
        }

        return params;
    }

    unimplemented!("Only objects are supported with style=deepObject")
}

pub mod alerts_api;
pub mod analytics_api;
pub mod appliance_configs_api;
pub mod attachments_api;
pub mod audit_events_api;
pub mod buckets_api;
pub mod comments_api;
pub mod conversational_filter_api;
pub mod dashboards_api;
pub mod datasets_api;
pub mod deployment_api;
pub mod deprecation_api;
pub mod discover_api;
pub mod documents_api;
pub mod emails_api;
pub mod files_api;
pub mod integrations_api;
pub mod ixp_datasets_api;
pub mod ixp_models_api;
pub mod ixp_projects_api;
pub mod label_defs_api;
pub mod label_groups_api;
pub mod metadata_api;
pub mod model_family_api;
pub mod models_api;
pub mod permissions_api;
pub mod projects_api;
pub mod quotas_api;
pub mod reports_api;
pub mod search_api;
pub mod sources_api;
pub mod streams_api;
pub mod tenants_api;
pub mod themes_api;
pub mod thread_themes_api;
pub mod triggers_api;
pub mod uipath_provisioning_api;
pub mod users_api;

pub mod configuration;
