//! Project configuration management (OIDC, SAML).

use crate::auth::project_config::{
    CreateOidcProviderConfigRequest, CreateSamlProviderConfigRequest,
    ListOidcProviderConfigsResponse, ListSamlProviderConfigsResponse, OidcProviderConfig,
    SamlProviderConfig, UpdateOidcProviderConfigRequest, UpdateSamlProviderConfigRequest,
};
use crate::auth::AuthError;
use crate::core::middleware::AuthMiddleware;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use url::Url;

const IDENTITY_TOOLKIT_URL: &str = "https://identitytoolkit.googleapis.com/v2";

/// Manages project-level configurations like OIDC and SAML providers.
#[derive(Clone)]
pub struct ProjectConfig {
    client: ClientWithMiddleware,
    base_url: String,
}

impl ProjectConfig {
    pub(crate) fn new(middleware: AuthMiddleware) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(middleware.clone())
            .build();

        let project_id = middleware.key.project_id.clone().unwrap_or_default();
        let base_url = format!("{}/projects/{}", IDENTITY_TOOLKIT_URL, project_id);

        Self { client, base_url }
    }

    // --- OIDC Provider Configs ---

    pub async fn create_oidc_provider_config(
        &self,
        request: CreateOidcProviderConfigRequest,
    ) -> Result<OidcProviderConfig, AuthError> {
        let url = format!("{}/oauthIdpConfigs", self.base_url);
        let mut url_obj = Url::parse(&url).map_err(|e| AuthError::ApiError(e.to_string()))?;
        url_obj.query_pairs_mut().append_pair("oauthIdpConfigId", &request.oauth_idp_config_id);

        let response = self
            .client
            .post(url_obj)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Create OIDC config failed {}: {}",
                status, text
            )));
        }

        let config: OidcProviderConfig = response.json().await?;
        Ok(config)
    }

    pub async fn get_oidc_provider_config(
        &self,
        config_id: &str,
    ) -> Result<OidcProviderConfig, AuthError> {
        let url = format!("{}/oauthIdpConfigs/{}", self.base_url, config_id);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Get OIDC config failed {}: {}",
                status, text
            )));
        }

        let config: OidcProviderConfig = response.json().await?;
        Ok(config)
    }

    pub async fn update_oidc_provider_config(
        &self,
        config_id: &str,
        request: UpdateOidcProviderConfigRequest,
    ) -> Result<OidcProviderConfig, AuthError> {
        let url = format!("{}/oauthIdpConfigs/{}", self.base_url, config_id);

        let mut mask_parts = Vec::new();
        if request.display_name.is_some() { mask_parts.push("displayName"); }
        if request.enabled.is_some() { mask_parts.push("enabled"); }
        if request.client_id.is_some() { mask_parts.push("clientId"); }
        if request.issuer.is_some() { mask_parts.push("issuer"); }
        if request.client_secret.is_some() { mask_parts.push("clientSecret"); }
        if request.response_type.is_some() { mask_parts.push("responseType"); }

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
                "Update OIDC config failed {}: {}",
                status, text
            )));
        }

        let config: OidcProviderConfig = response.json().await?;
        Ok(config)
    }

    pub async fn delete_oidc_provider_config(&self, config_id: &str) -> Result<(), AuthError> {
        let url = format!("{}/oauthIdpConfigs/{}", self.base_url, config_id);

        let response = self.client.delete(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Delete OIDC config failed {}: {}",
                status, text
            )));
        }

        Ok(())
    }

    pub async fn list_oidc_provider_configs(
        &self,
        max_results: Option<u32>,
        page_token: Option<&str>,
    ) -> Result<ListOidcProviderConfigsResponse, AuthError> {
        let url = format!("{}/oauthIdpConfigs", self.base_url);
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
                "List OIDC configs failed {}: {}",
                status, text
            )));
        }

        let result: ListOidcProviderConfigsResponse = response.json().await?;
        Ok(result)
    }

    // --- SAML Provider Configs ---

    pub async fn create_saml_provider_config(
        &self,
        request: CreateSamlProviderConfigRequest,
    ) -> Result<SamlProviderConfig, AuthError> {
        let url = format!("{}/inboundSamlConfigs", self.base_url);
        let mut url_obj = Url::parse(&url).map_err(|e| AuthError::ApiError(e.to_string()))?;
        url_obj.query_pairs_mut().append_pair("inboundSamlConfigId", &request.inbound_saml_config_id);

        let response = self
            .client
            .post(url_obj)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Create SAML config failed {}: {}",
                status, text
            )));
        }

        let config: SamlProviderConfig = response.json().await?;
        Ok(config)
    }

    pub async fn get_saml_provider_config(
        &self,
        config_id: &str,
    ) -> Result<SamlProviderConfig, AuthError> {
        let url = format!("{}/inboundSamlConfigs/{}", self.base_url, config_id);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Get SAML config failed {}: {}",
                status, text
            )));
        }

        let config: SamlProviderConfig = response.json().await?;
        Ok(config)
    }

    pub async fn update_saml_provider_config(
        &self,
        config_id: &str,
        request: UpdateSamlProviderConfigRequest,
    ) -> Result<SamlProviderConfig, AuthError> {
        let url = format!("{}/inboundSamlConfigs/{}", self.base_url, config_id);

        let mut mask_parts = Vec::new();
        if request.display_name.is_some() { mask_parts.push("displayName"); }
        if request.enabled.is_some() { mask_parts.push("enabled"); }

        // Nested fields need to be handled carefully for mask
        if let Some(idp) = &request.idp_config {
            if idp.idp_entity_id.is_some() { mask_parts.push("idpConfig.idpEntityId"); }
            if idp.sso_url.is_some() { mask_parts.push("idpConfig.ssoUrl"); }
            if idp.sign_request.is_some() { mask_parts.push("idpConfig.signRequest"); }
            if idp.idp_certificates.is_some() { mask_parts.push("idpConfig.idpCertificates"); }
        }

        if let Some(sp) = &request.sp_config {
            if sp.sp_entity_id.is_some() { mask_parts.push("spConfig.spEntityId"); }
            if sp.callback_uri.is_some() { mask_parts.push("spConfig.callbackUri"); }
        }

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
                "Update SAML config failed {}: {}",
                status, text
            )));
        }

        let config: SamlProviderConfig = response.json().await?;
        Ok(config)
    }

    pub async fn delete_saml_provider_config(&self, config_id: &str) -> Result<(), AuthError> {
        let url = format!("{}/inboundSamlConfigs/{}", self.base_url, config_id);

        let response = self.client.delete(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Delete SAML config failed {}: {}",
                status, text
            )));
        }

        Ok(())
    }

    pub async fn list_saml_provider_configs(
        &self,
        max_results: Option<u32>,
        page_token: Option<&str>,
    ) -> Result<ListSamlProviderConfigsResponse, AuthError> {
        let url = format!("{}/inboundSamlConfigs", self.base_url);
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
                "List SAML configs failed {}: {}",
                status, text
            )));
        }

        let result: ListSamlProviderConfigsResponse = response.json().await?;
        Ok(result)
    }
}