//! Data models for Firebase Authentication.

use serde::{Deserialize, Serialize};

/// Represents a user in the Firebase project.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserRecord {
    /// The user's unique ID.
    pub local_id: String,
    /// The user's email address.
    pub email: Option<String>,
    /// Whether the user's email has been verified.
    pub email_verified: bool,
    /// The user's display name.
    pub display_name: Option<String>,
    /// The user's photo URL.
    pub photo_url: Option<String>,
    /// The user's phone number.
    pub phone_number: Option<String>,
    /// Whether the user is disabled.
    pub disabled: bool,
    /// Additional metadata about the user.
    pub metadata: Option<UserMetadata>,
    /// Information about the user's providers (Google, Facebook, etc.).
    pub provider_user_info: Option<Vec<ProviderUserInfo>>,
    /// The user's password hash.
    pub password_hash: Option<String>,
    /// The user's password salt.
    pub password_salt: Option<String>,
    /// Custom claims set on the user (JSON string).
    pub custom_attributes: Option<String>,
    /// The user's tenant ID (for multi-tenancy).
    pub tenant_id: Option<String>,
    /// Multi-factor authentication info.
    pub mfa_info: Option<Vec<MfaInfo>>,
}

/// Metadata associated with a user account.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserMetadata {
    /// The date and time the user last signed in.
    pub last_sign_in_time: Option<String>,
    /// The date and time the account was created.
    pub creation_time: Option<String>,
    /// The date and time the user last refreshed their token.
    pub last_refresh_time: Option<String>,
}

/// Information about a user's identity provider.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUserInfo {
    /// The ID of the identity provider (e.g., google.com).
    pub provider_id: String,
    /// The user's display name linked to this provider.
    pub display_name: Option<String>,
    /// The user's photo URL linked to this provider.
    pub photo_url: Option<String>,
    /// The user's federated ID.
    pub federated_id: Option<String>,
    /// The user's email linked to this provider.
    pub email: Option<String>,
    /// The user's raw ID.
    pub raw_id: Option<String>,
    /// The user's screen name.
    pub screen_name: Option<String>,
}

/// Multi-factor authentication information.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MfaInfo {
    /// The MFA enrollment ID.
    pub mfa_enrollment_id: Option<String>,
    /// The display name for this MFA method.
    pub display_name: Option<String>,
    /// The phone number info for this MFA method.
    pub phone_info: Option<String>,
    /// The date and time this MFA method was enrolled.
    pub enrolled_at: Option<String>,
}

/// Request to create a new user.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    /// The UID to assign to the new user. If not provided, one will be generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_id: Option<String>,
    /// The user's email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Whether the user's email is verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    /// The user's password.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// The user's display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The user's photo URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    /// Whether the user is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// The user's phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
}

/// Request to update an existing user.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    /// The UID of the user to update.
    pub local_id: String,
    /// The new email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The new email verification status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    /// The new password.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// The new display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The new photo URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    /// The new disabled status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// The new phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// The new custom claims (JSON string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_attributes: Option<String>,
    /// Force token expiration (set validSince to now).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_since: Option<String>,
    /// List of attributes to delete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_attribute: Option<Vec<String>>,
    /// List of providers to unlink.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_provider: Option<Vec<String>>,
}

/// Response from listing users.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersResponse {
    /// The list of users.
    pub users: Option<Vec<UserRecord>>,
    /// The token for the next page of results.
    pub next_page_token: Option<String>,
}

/// Internal request to get account info.
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

/// Internal response from getting account info.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountInfoResponse {
    pub users: Option<Vec<UserRecord>>,
}

/// Internal request to delete an account.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAccountRequest {
    pub local_id: String,
}

/// Internal request for email actions.
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
    #[serde(rename = "iOSBundleId", skip_serializing_if = "Option::is_none")]
    pub ios_bundle_id: Option<String>,
}

/// Internal response for email actions.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailLinkResponse {
    pub email: Option<String>,
    pub oob_link: String,
}

/// A user record used for bulk import.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserImportRecord {
    /// The user's UID.
    pub local_id: String,
    /// The user's email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Whether the user's email is verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    /// The user's password hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    /// The user's password salt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_salt: Option<String>,
    /// The user's display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The user's photo URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    /// Whether the user is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// The user's phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// The user's custom claims (JSON string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_attributes: Option<String>,
}

/// Request to import users in bulk.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportUsersRequest {
    /// The list of users to import.
    pub users: Vec<UserImportRecord>,
    /// The hashing algorithm used for passwords (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<UserImportHash>,
}

/// Password hashing configuration for user import.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserImportHash {
    /// The hashing algorithm (e.g., "SCRYPT").
    pub hash_algorithm: String,
    /// The signing key (base64 encoded).
    pub key: String,
    /// The salt separator (base64 encoded).
    pub salt_separator: String,
    /// The number of rounds.
    pub rounds: i32,
    /// The memory cost.
    pub memory_cost: i32,
}

/// Response from user import.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportUsersResponse {
    /// List of errors encountered during import.
    pub error: Option<Vec<ImportUserError>>,
}

/// Error detail for a failed user import.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportUserError {
    /// The index of the user in the request list.
    pub index: usize,
    /// The error message.
    pub message: String,
}

/// Request to create a session cookie.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionCookieRequest {
    /// The ID token to exchange for a session cookie.
    pub id_token: String,
    /// The number of seconds until the session cookie expires.
    #[serde(rename = "validDuration")]
    pub valid_duration_seconds: u64,
}

/// Response from creating a session cookie.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionCookieResponse {
    /// The created session cookie.
    pub session_cookie: String,
}

// --- Action Code Settings ---

/// Settings for generating email action links.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionCodeSettings {
    /// The URL to continue to after the user clicks the link.
    pub url: String,
    /// Whether to open the link via a mobile app if installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle_code_in_app: Option<bool>,
    /// iOS specific settings.
    #[serde(rename = "iOS", skip_serializing_if = "Option::is_none")]
    pub ios: Option<IosSettings>,
    /// Android specific settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android: Option<AndroidSettings>,
    /// The dynamic link domain to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_link_domain: Option<String>,
}

/// iOS specific settings for action code.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IosSettings {
    /// The iOS bundle ID.
    pub bundle_id: String,
}

/// Android specific settings for action code.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AndroidSettings {
    /// The Android package name.
    pub package_name: String,
    /// Whether to install the app if not already installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_app: Option<bool>,
    /// The minimum version of the app required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_version: Option<String>,
}
