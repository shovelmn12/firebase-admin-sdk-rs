use serde::{Deserialize, Serialize};

/// Represents an OIDC Provider Configuration.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OidcProviderConfig {
    /// The resource name of the config.
    /// Format: "projects/{project-id}/oauthIdpConfigs/{config-id}"
    pub name: String,

    /// The display name for this provider.
    pub display_name: Option<String>,

    /// Whether this provider is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// The client ID for the OIDC provider.
    pub client_id: Option<String>,

    /// The issuer URL for the OIDC provider.
    pub issuer: Option<String>,

    /// The client secret for the OIDC provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// The response type (e.g., "code", "id_token").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_type: Option<OidcResponseType>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OidcResponseType {
    /// Whether the ID token is requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<bool>,
    /// Whether the code is requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<bool>,
}

/// Request to create an OIDC Provider Config.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateOidcProviderConfigRequest {
    /// The ID to use for the new config.
    #[serde(skip)]
    pub oauth_idp_config_id: String,

    pub display_name: Option<String>,
    pub enabled: Option<bool>,
    pub client_id: String,
    pub issuer: String,
    pub client_secret: Option<String>,
    pub response_type: Option<OidcResponseType>,
}

/// Request to update an OIDC Provider Config.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOidcProviderConfigRequest {
    pub display_name: Option<String>,
    pub enabled: Option<bool>,
    pub client_id: Option<String>,
    pub issuer: Option<String>,
    pub client_secret: Option<String>,
    pub response_type: Option<OidcResponseType>,
}

/// Response from listing OIDC Provider Configs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOidcProviderConfigsResponse {
    pub oauth_idp_configs: Option<Vec<OidcProviderConfig>>,
    pub next_page_token: Option<String>,
}

// --- SAML Structures ---

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SamlIdpConfig {
    pub idp_entity_id: Option<String>,
    pub sso_url: Option<String>,
    pub sign_request: Option<bool>,
    pub idp_certificates: Option<Vec<SamlCertificate>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SamlSpConfig {
    pub sp_entity_id: Option<String>,
    pub callback_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SamlCertificate {
    pub x509_certificate: String,
}

/// Represents a SAML Provider Configuration.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SamlProviderConfig {
    /// The resource name of the config.
    pub name: String,

    /// The display name.
    pub display_name: Option<String>,

    /// Whether enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// IDP configuration.
    pub idp_config: Option<SamlIdpConfig>,

    /// SP configuration.
    pub sp_config: Option<SamlSpConfig>,
}

/// Request to create a SAML Provider Config.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateSamlProviderConfigRequest {
    /// The ID to use for the new config.
    #[serde(skip)]
    pub inbound_saml_config_id: String,

    pub display_name: Option<String>,
    pub enabled: Option<bool>,
    pub idp_config: SamlIdpConfig,
    pub sp_config: SamlSpConfig,
}

/// Request to update a SAML Provider Config.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSamlProviderConfigRequest {
    pub display_name: Option<String>,
    pub enabled: Option<bool>,
    pub idp_config: Option<SamlIdpConfig>,
    pub sp_config: Option<SamlSpConfig>,
}

/// Response from listing SAML Provider Configs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSamlProviderConfigsResponse {
    pub inbound_saml_configs: Option<Vec<SamlProviderConfig>>,
    pub next_page_token: Option<String>,
}
