use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{
    error::{Error, Result},
    resources::user::Id as UserId,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ProjectName(pub String);

impl FromStr for ProjectName {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            Ok(Self(string.into()))
        } else {
            Err(Error::BadProjectIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Project {
    pub name: ProjectName,
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetProjectResponse {
    pub project: Project,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetProjectsResponse {
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct NewProject<'request> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub(crate) struct CreateProjectRequest<'request> {
    pub project: NewProject<'request>,
    pub user_ids: &'request [UserId],
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CreateProjectResponse {
    pub project: Project,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct UpdateProject<'request> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct UpdateProjectRequest<'request> {
    pub project: UpdateProject<'request>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct UpdateProjectResponse {
    pub project: Project,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceDeleteProject {
    No,
    Yes,
}
