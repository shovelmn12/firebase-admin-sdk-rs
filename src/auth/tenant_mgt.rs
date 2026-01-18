//! Tenant management module.

use crate::auth::{AuthError, FirebaseAuth};
use crate::core::middleware::AuthMiddleware;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use url::Url;

const IDENTITY_TOOLKIT_URL: &str = "https://identitytoolkit.googleapis.com/v2";

/// Represents a tenant in a multi-tenant project.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Tenant {
    /// The resource name of the tenant.
    /// Format: "projects/{project-id}/tenants/{tenant-id}"
    pub name: String,

    /// The display name of the tenant.
    pub display_name: Option<String>,

    /// Whether to allow email/password user authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_password_signup: Option<bool>,

    /// Whether to enable email link user authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_email_link_signin: Option<bool>,

    /// Whether authentication is disabled for the tenant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_auth: Option<bool>,

    /// Whether to enable anonymous user authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_anonymous_user: Option<bool>,

    /// Map of test phone numbers and their fake verification codes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_phone_numbers: Option<std::collections::HashMap<String, String>>,

    /// The tenant-level configuration of MFA options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_config: Option<serde_json::Value>,

    /// The tenant-level reCAPTCHA config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recaptcha_config: Option<serde_json::Value>,

    /// Configures which regions are enabled for SMS verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms_region_config: Option<serde_json::Value>,

    /// Configuration related to monitoring project activity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring: Option<serde_json::Value>,

    /// The tenant-level password policy config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_policy_config: Option<serde_json::Value>,

    /// Configuration for settings related to email privacy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_privacy_config: Option<serde_json::Value>,

    /// Options related to how clients making requests on behalf of a project should be configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<serde_json::Value>,
}

/// Request to create a new tenant.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateTenantRequest {
    /// The display name of the tenant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Whether to allow email/password user authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_password_signup: Option<bool>,

    /// Whether to enable email link user authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_email_link_signin: Option<bool>,

    /// Whether authentication is disabled for the tenant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_auth: Option<bool>,

    /// Whether to enable anonymous user authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_anonymous_user: Option<bool>,

    /// Map of test phone numbers and their fake verification codes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_phone_numbers: Option<std::collections::HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_config: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub recaptcha_config: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms_region_config: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_policy_config: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_privacy_config: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<serde_json::Value>,
}

/// Request to update a tenant.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTenantRequest {
    /// The display name of the tenant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Whether to allow email/password user authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_password_signup: Option<bool>,

    /// Whether to enable email link user authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_email_link_signin: Option<bool>,

    /// Whether authentication is disabled for the tenant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_auth: Option<bool>,

    /// Whether to enable anonymous user authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_anonymous_user: Option<bool>,

    /// Map of test phone numbers and their fake verification codes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_phone_numbers: Option<std::collections::HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_config: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub recaptcha_config: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms_region_config: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_policy_config: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_privacy_config: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<serde_json::Value>,
}

/// Response from listing tenants.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTenantsResponse {
    /// The list of tenants.
    pub tenants: Option<Vec<Tenant>>,
    /// The token for the next page of results.
    pub next_page_token: Option<String>,
}

/// Manages tenants in a multi-tenant project.
#[derive(Clone)]
pub struct TenantAwareness {
    client: ClientWithMiddleware,
    base_url: String,
    middleware: AuthMiddleware,
}

impl TenantAwareness {
    pub(crate) fn new(middleware: AuthMiddleware) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(middleware.clone())
            .build();

        let project_id = middleware.key.project_id.clone().unwrap_or_default();
        let base_url = format!("{}/projects/{}", IDENTITY_TOOLKIT_URL, project_id);

        Self {
            client,
            base_url,
            middleware,
        }
    }

    /// Returns a `FirebaseAuth` instance scoped to the specified tenant.
    pub fn auth_for_tenant(&self, tenant_id: &str) -> FirebaseAuth {
        let middleware = self.middleware.with_tenant(tenant_id);
        FirebaseAuth::new(middleware)
    }

    /// Creates a new tenant.
    pub async fn create_tenant(&self, request: CreateTenantRequest) -> Result<Tenant, AuthError> {
        let url = format!("{}/tenants", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Create tenant failed {}: {}",
                status, text
            )));
        }

        let tenant: Tenant = response.json().await?;
        Ok(tenant)
    }

    /// Retrieves a tenant by ID.
    pub async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant, AuthError> {
        let url = format!("{}/tenants/{}", self.base_url, tenant_id);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Get tenant failed {}: {}",
                status, text
            )));
        }

        let tenant: Tenant = response.json().await?;
        Ok(tenant)
    }

    /// Updates a tenant.
    pub async fn update_tenant(
        &self,
        tenant_id: &str,
        request: UpdateTenantRequest,
    ) -> Result<Tenant, AuthError> {
        let url = format!("{}/tenants/{}", self.base_url, tenant_id);

        let mut mask_parts = Vec::new();
        if request.display_name.is_some() { mask_parts.push("displayName"); }
        if request.allow_password_signup.is_some() { mask_parts.push("allowPasswordSignup"); }
        if request.enable_email_link_signin.is_some() { mask_parts.push("enableEmailLinkSignin"); }
        if request.disable_auth.is_some() { mask_parts.push("disableAuth"); }
        if request.enable_anonymous_user.is_some() { mask_parts.push("enableAnonymousUser"); }
        if request.test_phone_numbers.is_some() { mask_parts.push("testPhoneNumbers"); }
        if request.mfa_config.is_some() { mask_parts.push("mfaConfig"); }
        if request.recaptcha_config.is_some() { mask_parts.push("recaptchaConfig"); }
        if request.sms_region_config.is_some() { mask_parts.push("smsRegionConfig"); }
        if request.monitoring.is_some() { mask_parts.push("monitoring"); }
        if request.password_policy_config.is_some() { mask_parts.push("passwordPolicyConfig"); }
        if request.email_privacy_config.is_some() { mask_parts.push("emailPrivacyConfig"); }
        if request.client.is_some() { mask_parts.push("client"); }

        let update_mask = mask_parts.join(",");

        let mut url_obj = Url::parse(&url).map_err(|e| AuthError::ApiError(e.to_string()))?;
        url_obj.query_pairs_mut().append_pair("updateMask", &update_mask);

        let response = self
            .client
            .patch(url_obj)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Update tenant failed {}: {}",
                status, text
            )));
        }

        let tenant: Tenant = response.json().await?;
        Ok(tenant)
    }

    /// Deletes a tenant.
    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<(), AuthError> {
        let url = format!("{}/tenants/{}", self.base_url, tenant_id);

        let response = self.client.delete(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Delete tenant failed {}: {}",
                status, text
            )));
        }

        Ok(())
    }

    /// Lists tenants.
    pub async fn list_tenants(
        &self,
        max_results: Option<u32>,
        page_token: Option<&str>,
    ) -> Result<ListTenantsResponse, AuthError> {
        let url = format!("{}/tenants", self.base_url);
        let mut url_obj = Url::parse(&url).map_err(|e| AuthError::ApiError(e.to_string()))?;

        {
            let mut query_pairs = url_obj.query_pairs_mut();
            if let Some(max) = max_results {
                query_pairs.append_pair("pageSize", &max.to_string());
            }
            if let Some(token) = page_token {
                query_pairs.append_pair("pageToken", token);
            }
        }

        let response = self.client.get(url_obj).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "List tenants failed {}: {}",
                status, text
            )));
        }

        let result: ListTenantsResponse = response.json().await?;
        Ok(result)
    }
}
