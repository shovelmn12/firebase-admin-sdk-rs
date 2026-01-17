//! Firebase Authentication module.
//!
//! This module provides functionality for managing users (create, update, delete, list, get)
//! and generating OOB (Out-of-Band) codes for email actions like password resets and email verification.
//! It also includes ID token verification.

pub mod keys;
pub mod models;
pub mod verifier;
pub mod tenant_mgt;
pub mod project_config;
pub mod project_config_impl;

use crate::auth::models::{
    CreateSessionCookieRequest, CreateSessionCookieResponse, CreateUserRequest,
    DeleteAccountRequest, EmailLinkRequest, EmailLinkResponse, GetAccountInfoRequest,
    GetAccountInfoResponse, ImportUsersRequest, ImportUsersResponse, ListUsersResponse,
    UpdateUserRequest, UserRecord,
};
use crate::auth::verifier::{FirebaseTokenClaims, IdTokenVerifier, TokenVerificationError};
use crate::auth::tenant_mgt::TenantAwareness;
use crate::auth::project_config_impl::ProjectConfig;
use crate::core::middleware::AuthMiddleware;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::header;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use url::Url;

const AUTH_V1_API: &str = "https://identitytoolkit.googleapis.com/v1/projects/{project_id}";
const AUTH_V1_TENANT_API: &str = "https://identitytoolkit.googleapis.com/v1/projects/{project_id}/tenants/{tenant_id}";

/// Errors that can occur during Authentication operations.
#[derive(Error, Debug)]
pub enum AuthError {
    /// Wrapper for `reqwest::Error`.
    #[error("HTTP Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    /// Wrapper for `reqwest_middleware::Error`.
    #[error("Middleware error: {0}")]
    MiddlewareError(#[from] reqwest_middleware::Error),
    /// Errors returned by the Identity Toolkit API.
    #[error("API error: {0}")]
    ApiError(String),
    /// The requested user was not found.
    #[error("User not found")]
    UserNotFound,
    /// Wrapper for `serde_json::Error`.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    /// Error during ID token verification.
    #[error("Token verification error: {0}")]
    TokenVerificationError(#[from] TokenVerificationError),
    /// Wrapper for `jsonwebtoken::errors::Error`.
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    /// The private key provided in the service account is invalid.
    #[error("Invalid private key")]
    InvalidPrivateKey,
    /// A service account key is required for this operation (e.g., custom token signing) but was not provided.
    #[error("Service account key required for this operation")]
    ServiceAccountKeyRequired,
    /// Errors occurred during a bulk import operation.
    #[error("Import users error: {0:?}")]
    ImportUsersError(Vec<models::ImportUserError>),
}

/// Claims used for generating custom tokens.
#[derive(Debug, Serialize)]
struct CustomTokenClaims {
    iss: String,
    sub: String,
    aud: String,
    iat: usize,
    exp: usize,
    uid: String,
    #[serde(flatten)]
    claims: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Client for interacting with Firebase Authentication.
#[derive(Clone)]
pub struct FirebaseAuth {
    client: ClientWithMiddleware,
    base_url: String,
    verifier: Arc<IdTokenVerifier>,
    middleware: AuthMiddleware,
    tenant_id: Option<String>,
}

impl FirebaseAuth {
    /// Creates a new `FirebaseAuth` instance.
    ///
    /// This is typically called via `FirebaseApp::auth()`.
    pub fn new(middleware: AuthMiddleware) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(middleware.clone())
            .build();

        let key = &middleware.key;
        let project_id = key.project_id.clone().unwrap_or_default();
        let verifier = Arc::new(IdTokenVerifier::new(project_id.clone()));

        let tenant_id = middleware.tenant_id();

        let base_url = if let Some(tid) = &tenant_id {
             AUTH_V1_TENANT_API.replace("{project_id}", &project_id).replace("{tenant_id}", tid)
        } else {
             AUTH_V1_API.replace("{project_id}", &project_id)
        };

        Self {
            client,
            base_url,
            verifier,
            middleware,
            tenant_id,
        }
    }

    /// Returns the tenant awareness interface.
    pub fn tenant_manager(&self) -> TenantAwareness {
        TenantAwareness::new(self.middleware.clone())
    }

    /// Returns the project config interface.
    pub fn project_config_manager(&self) -> ProjectConfig {
        ProjectConfig::new(self.middleware.clone())
    }

    /// Verifies a Firebase ID token.
    ///
    /// This method fetches Google's public keys (caching them respecting Cache-Control)
    /// and verifies the signature, audience, issuer, and expiration of the token.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT ID token string.
    pub async fn verify_id_token(&self, token: &str) -> Result<FirebaseTokenClaims, AuthError> {
        Ok(self.verifier.verify_id_token(token).await?)
    }

    /// Creates a session cookie from an ID token.
    ///
    /// # Arguments
    ///
    /// * `id_token` - The ID token to exchange for a session cookie.
    /// * `valid_duration` - The duration for which the session cookie is valid.
    pub async fn create_session_cookie(
        &self,
        id_token: &str,
        valid_duration: std::time::Duration,
    ) -> Result<String, AuthError> {
        let url = format!("{}:createSessionCookie", self.base_url);

        let request = CreateSessionCookieRequest {
            id_token: id_token.to_string(),
            valid_duration_seconds: valid_duration.as_secs(),
        };

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Create session cookie failed {}: {}",
                status, text
            )));
        }

        let result: CreateSessionCookieResponse = response.json().await?;
        Ok(result.session_cookie)
    }

    /// Verifies a Firebase session cookie.
    ///
    /// # Arguments
    ///
    /// * `session_cookie` - The session cookie string.
    pub async fn verify_session_cookie(
        &self,
        session_cookie: &str,
    ) -> Result<FirebaseTokenClaims, AuthError> {
        Ok(self.verifier.verify_session_cookie(session_cookie).await?)
    }

    /// Creates a custom token for the given UID with optional custom claims.
    ///
    /// This token can be sent to a client application to sign in with `signInWithCustomToken`.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier for the user.
    /// * `custom_claims` - Optional JSON object containing custom claims.
    pub fn create_custom_token(
        &self,
        uid: &str,
        custom_claims: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<String, AuthError> {
        let key = &self.middleware.key;
        let client_email = key.client_email.clone();
        let private_key = key.private_key.clone();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let mut final_claims = custom_claims.unwrap_or_default();
        if let Some(tid) = &self.tenant_id {
            final_claims.insert("tenant_id".to_string(), serde_json::Value::String(tid.clone()));
        }

        let claims = CustomTokenClaims {
            iss: client_email.clone(),
            sub: client_email,
            aud: "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit".to_string(),
            iat: now,
            exp: now + 3600, // 1 hour expiration
            uid: uid.to_string(),
            claims: Some(final_claims),
        };

        let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes())
            .map_err(|_| AuthError::InvalidPrivateKey)?;

        let header = Header::new(Algorithm::RS256);
        let token = encode(&header, &claims, &encoding_key)?;

        Ok(token)
    }

    /// Internal helper to generate OOB (Out-of-Band) email links.
    async fn generate_email_link(
        &self,
        request_type: &str,
        email: &str,
        settings: Option<serde_json::Value>,
    ) -> Result<String, AuthError> {
        let url = format!("{}/accounts:sendOobCode", self.base_url,);

        // Need to map generic settings to EmailLinkRequest
        let mut request = EmailLinkRequest {
            request_type: request_type.to_string(),
            email: Some(email.to_string()),
            ..Default::default()
        };

        if let Some(s) = settings {
            // Simplistic mapping for now, ideally pass a struct
            if let Some(url) = s.get("continueUrl").and_then(|v| v.as_str()) {
                request.continue_url = Some(url.to_string());
            }
            // ... map other fields
        }

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Generate email link failed {}: {}",
                status, text
            )));
        }

        let result: EmailLinkResponse = response.json().await?;
        Ok(result.oob_link)
    }

    /// Generates a link for password reset.
    pub async fn generate_password_reset_link(
        &self,
        email: &str,
        settings: Option<serde_json::Value>,
    ) -> Result<String, AuthError> {
        self.generate_email_link("PASSWORD_RESET", email, settings)
            .await
    }

    /// Generates a link for email verification.
    pub async fn generate_email_verification_link(
        &self,
        email: &str,
        settings: Option<serde_json::Value>,
    ) -> Result<String, AuthError> {
        self.generate_email_link("VERIFY_EMAIL", email, settings)
            .await
    }

    /// Generates a link for sign-in with email.
    pub async fn generate_sign_in_with_email_link(
        &self,
        email: &str,
        settings: Option<serde_json::Value>,
    ) -> Result<String, AuthError> {
        self.generate_email_link("EMAIL_SIGNIN", email, settings)
            .await
    }

    /// Imports users in bulk.
    ///
    /// # Arguments
    ///
    /// * `request` - An `ImportUsersRequest` containing the list of users and hashing algorithm configuration.
    pub async fn import_users(
        &self,
        request: ImportUsersRequest,
    ) -> Result<ImportUsersResponse, AuthError> {
        let url = format!("{}/accounts:batchCreate", self.base_url,);

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Import users failed {}: {}",
                status, text
            )));
        }

        let result: ImportUsersResponse = response.json().await?;

        if let Some(errors) = &result.error {
            if !errors.is_empty() {
                // Partial failure or full failure reporting depending on API behavior
                // Usually batchCreate returns 200 with errors list for partials.
                // We can return the response or error out.
                // Let's return the response but user should check it.
                // Or we can define that if errors exist, we return Err(AuthError::ImportUsersError(errors))
                return Err(AuthError::ImportUsersError(
                    errors
                        .iter()
                        .map(|e| models::ImportUserError {
                            index: e.index,
                            message: e.message.clone(),
                        })
                        .collect(),
                ));
            }
        }

        Ok(result)
    }

    /// Creates a new user.
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserRecord, AuthError> {
        let url = format!("{}/accounts", self.base_url);

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Create user failed {}: {}",
                status, text
            )));
        }

        let user: UserRecord = response.json().await?;
        Ok(user)
    }

    /// Updates an existing user.
    pub async fn update_user(&self, request: UpdateUserRequest) -> Result<UserRecord, AuthError> {
        let url = format!("{}/accounts:update", self.base_url);

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Update user failed {}: {}",
                status, text
            )));
        }

        let user: UserRecord = response.json().await?;
        Ok(user)
    }

    /// Deletes a user by UID.
    pub async fn delete_user(&self, uid: &str) -> Result<(), AuthError> {
        let url = format!("{}/accounts:delete", self.base_url);
        let request = DeleteAccountRequest {
            local_id: uid.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Delete user failed {}: {}",
                status, text
            )));
        }

        Ok(())
    }

    /// Internal helper to get account info.
    async fn get_account_info(
        &self,
        request: GetAccountInfoRequest,
    ) -> Result<UserRecord, AuthError> {
        let url = format!("{}/accounts:lookup", self.base_url);

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "Get user failed {}: {}",
                status, text
            )));
        }

        let result: GetAccountInfoResponse = response.json().await?;

        result
            .users
            .and_then(|mut users| users.pop())
            .ok_or(AuthError::UserNotFound)
    }

    /// Retrieves a user by their UID.
    pub async fn get_user(&self, uid: &str) -> Result<UserRecord, AuthError> {
        let request = GetAccountInfoRequest {
            local_id: Some(vec![uid.to_string()]),
            email: None,
            phone_number: None,
        };
        self.get_account_info(request).await
    }

    /// Retrieves a user by their email.
    pub async fn get_user_by_email(&self, email: &str) -> Result<UserRecord, AuthError> {
        let request = GetAccountInfoRequest {
            local_id: None,
            email: Some(vec![email.to_string()]),
            phone_number: None,
        };
        self.get_account_info(request).await
    }

    /// Retrieves a user by their phone number.
    pub async fn get_user_by_phone_number(&self, phone: &str) -> Result<UserRecord, AuthError> {
        let request = GetAccountInfoRequest {
            local_id: None,
            email: None,
            phone_number: Some(vec![phone.to_string()]),
        };
        self.get_account_info(request).await
    }

    /// Lists users.
    ///
    /// # Arguments
    ///
    /// * `max_results` - The maximum number of users to return.
    /// * `page_token` - The next page token from a previous response.
    pub async fn list_users(
        &self,
        max_results: u32,
        page_token: Option<&str>,
    ) -> Result<ListUsersResponse, AuthError> {
        let url = format!("{}/accounts", self.base_url);
        let mut url_obj = Url::parse(&url).map_err(|e| AuthError::ApiError(e.to_string()))?;

        {
            let mut query_pairs = url_obj.query_pairs_mut();
            query_pairs.append_pair("maxResults", &max_results.to_string());
            if let Some(token) = page_token {
                query_pairs.append_pair("nextPageToken", token);
            }
        }

        let response = self.client.get(url_obj).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!(
                "List users failed {}: {}",
                status, text
            )));
        }

        let result: ListUsersResponse = response.json().await?;
        Ok(result)
    }
}
