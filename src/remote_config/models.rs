use std::collections::HashMap;

/// Represents a Remote Config template.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfig {
    /// The list of named conditions.
    #[serde(default)]
    pub conditions: Vec<RemoteConfigCondition>,
    /// The map of parameter keys to their optional default values and optional conditional values.
    #[serde(default)]
    pub parameters: HashMap<String, RemoteConfigParameter>,
    /// The map of parameter group names to their parameter group instances.
    #[serde(default)]
    pub parameter_groups: HashMap<String, RemoteConfigParameterGroup>,
    /// The ETag of the current template.
    #[serde(skip)]
    pub etag: String,
    /// Version information for the template.
    #[serde(default)]
    pub version: Option<Version>,
}

/// A condition that can be used to target specific users.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfigCondition {
    /// The name of the condition.
    pub name: String,
    /// The logic expression for the condition.
    pub expression: String,
    /// The color associated with the condition (for the console).
    pub tag_color: Option<String>,
}

/// A parameter in the Remote Config template.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfigParameter {
    /// The value to set the parameter to, if no conditions are met.
    pub default_value: Option<RemoteConfigParameterValue>,
    /// A map of condition names to values.
    #[serde(default)]
    pub conditional_values: HashMap<String, RemoteConfigParameterValue>,
    /// A description for the parameter.
    pub description: Option<String>,
}

/// The value of a Remote Config parameter.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum RemoteConfigParameterValue {
    /// A static string value.
    Value {
        /// The string value.
        value: String,
    },
    /// Indicates that the in-app default value should be used.
    UseInAppDefault {
        /// Always true if present.
        use_in_app_default: bool,
    },
}

/// A group of parameters.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfigParameterGroup {
    /// A description for the group.
    pub description: Option<String>,
    /// The parameters in the group.
    pub parameters: HashMap<String, RemoteConfigParameter>,
}

/// Version information for a Remote Config template.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    /// The version number.
    pub version_number: String,
    /// The time when this version was created.
    pub update_time: String,
    /// The user who created this version.
    pub update_user: Option<User>,
    /// A description of the version.
    pub description: Option<String>,
    /// The origin of the update (e.g. "CONSOLE").
    pub update_origin: String,
    /// The type of update (e.g. "INCREMENTAL_UPDATE").
    pub update_type: String,
}

/// User information.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// The user's email.
    pub email: String,
    /// The user's name.
    pub name: Option<String>,
    /// The user's image URL.
    pub image_url: Option<String>,
}

/// Options for listing versions.
#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVersionsOptions {
    /// The maximum number of versions to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<usize>,
    /// The token for the next page of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

/// The result of listing versions.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVersionsResult {
    /// The list of versions.
    pub versions: Vec<Version>,
    /// The token for the next page of results.
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

/// A request to rollback to a specific version.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RollbackRequest {
    /// The version number to roll back to.
    pub(crate) version_number: String,
}
