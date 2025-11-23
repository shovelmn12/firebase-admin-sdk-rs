use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfig {
    pub conditions: Vec<RemoteConfigCondition>,
    pub parameters: HashMap<String, RemoteConfigParameter>,
    #[serde(default)]
    pub parameter_groups: HashMap<String, RemoteConfigParameterGroup>,
    pub etag: String,
    #[serde(default)]
    pub version: Option<Version>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfigCondition {
    pub name: String,
    pub expression: String,
    pub tag_color: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfigParameter {
    pub default_value: Option<RemoteConfigParameterValue>,
    #[serde(default)]
    pub conditional_values: HashMap<String, RemoteConfigParameterValue>,
    pub description: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum RemoteConfigParameterValue {
    Value { value: String },
    UseInAppDefault { use_in_app_default: bool },
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfigParameterGroup {
    pub description: Option<String>,
    pub parameters: HashMap<String, RemoteConfigParameter>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub version_number: String,
    pub update_time: String,
    pub update_user: Option<User>,
    pub description: Option<String>,
    pub update_origin: String,
    pub update_type: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub email: String,
    pub name: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVersionsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVersionsResult {
    pub versions: Vec<Version>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RollbackRequest {
    pub(crate) version_number: String,
}
