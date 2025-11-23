use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserRecord {
    pub local_id: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub phone_number: Option<String>,
    pub disabled: bool,
    pub metadata: Option<UserMetadata>,
    pub provider_user_info: Option<Vec<ProviderUserInfo>>,
    pub password_hash: Option<String>,
    pub password_salt: Option<String>,
    pub custom_attributes: Option<String>, // JSON string for custom claims
    pub tenant_id: Option<String>,
    pub mfa_info: Option<Vec<MfaInfo>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserMetadata {
    pub last_sign_in_time: Option<String>,
    pub creation_time: Option<String>,
    pub last_refresh_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUserInfo {
    pub provider_id: String,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub federated_id: Option<String>,
    pub email: Option<String>,
    pub raw_id: Option<String>,
    pub screen_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MfaInfo {
    pub mfa_enrollment_id: Option<String>,
    pub display_name: Option<String>,
    pub phone_info: Option<String>,
    pub enrolled_at: Option<String>,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub local_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_attributes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_attribute: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_provider: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersResponse {
    pub users: Option<Vec<UserRecord>>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountInfoRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_id: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountInfoResponse {
    pub users: Option<Vec<UserRecord>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAccountRequest {
    pub local_id: String,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmailLinkRequest {
    pub request_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_handle_code_in_app: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_link_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android_package_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android_minimum_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android_install_app: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ios_bundle_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailLinkResponse {
    pub email: Option<String>,
    pub oob_link: String,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserImportRecord {
    pub local_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_salt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_attributes: Option<String>,
    // Additional fields like mfaInfo, tenantId can be added
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportUsersRequest {
    pub users: Vec<UserImportRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<UserImportHash>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserImportHash {
    pub hash_algorithm: String,
    pub key: String, // base64 encoded
    pub salt_separator: String, // base64 encoded
    pub rounds: i32,
    pub memory_cost: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportUsersResponse {
    pub error: Option<Vec<ImportUserError>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportUserError {
    pub index: usize,
    pub message: String,
}
