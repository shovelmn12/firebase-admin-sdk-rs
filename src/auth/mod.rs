pub mod models;
pub mod keys;
pub mod verifier;

use reqwest_middleware::ClientWithMiddleware;
use crate::auth::models::{
    CreateUserRequest, DeleteAccountRequest, GetAccountInfoRequest, GetAccountInfoResponse,
    ListUsersResponse, UpdateUserRequest, UserRecord, EmailLinkRequest, EmailLinkResponse,
    ImportUsersRequest, ImportUsersResponse,
};
use crate::auth::verifier::{IdTokenVerifier, FirebaseTokenClaims, TokenVerificationError};
use thiserror::Error;
use reqwest::header;
use std::sync::Arc;
use yup_oauth2::ServiceAccountKey;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("HTTP Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Middleware error: {0}")]
    MiddlewareError(#[from] reqwest_middleware::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("User not found")]
    UserNotFound,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Token verification error: {0}")]
    TokenVerificationError(#[from] TokenVerificationError),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Invalid private key")]
    InvalidPrivateKey,
    #[error("Service account key required for this operation")]
    ServiceAccountKeyRequired,
    #[error("Import users error: {0:?}")]
    ImportUsersError(Vec<crate::auth::models::ImportUserError>),
}

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

#[derive(Clone)]
pub struct FirebaseAuth {
    client: ClientWithMiddleware,
    project_id: String,
    verifier: Arc<IdTokenVerifier>,
    service_account_key: Option<ServiceAccountKey>,
}

impl FirebaseAuth {
    pub fn new(client: ClientWithMiddleware, project_id: String, service_account_key: Option<ServiceAccountKey>) -> Self {
        let verifier = Arc::new(IdTokenVerifier::new(project_id.clone()));
        Self { client, project_id, verifier, service_account_key }
    }

    // Base URL for Identity Toolkit API
    fn base_url(&self) -> String {
        "https://identitytoolkit.googleapis.com/v1/projects".to_string()
    }

    /// Verifies a Firebase ID token.
    pub async fn verify_id_token(&self, token: &str) -> Result<FirebaseTokenClaims, AuthError> {
        Ok(self.verifier.verify_token(token).await?)
    }

    /// Creates a custom token for the given UID with optional custom claims.
    pub fn create_custom_token(&self, uid: &str, custom_claims: Option<serde_json::Map<String, serde_json::Value>>) -> Result<String, AuthError> {
        let key = self.service_account_key.as_ref().ok_or(AuthError::ServiceAccountKeyRequired)?;
        let client_email = key.client_email.clone();
        let private_key = key.private_key.clone();

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;

        let claims = CustomTokenClaims {
            iss: client_email.clone(),
            sub: client_email,
            aud: "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit".to_string(),
            iat: now,
            exp: now + 3600, // 1 hour expiration
            uid: uid.to_string(),
            claims: custom_claims,
        };

        let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes())
            .map_err(|_| AuthError::InvalidPrivateKey)?;

        let header = Header::new(Algorithm::RS256);
        let token = encode(&header, &claims, &encoding_key)?;

        Ok(token)
    }

    async fn generate_email_link(&self, request_type: &str, email: &str, settings: Option<serde_json::Value>) -> Result<String, AuthError> {
         let url = format!("{}/{}/accounts:sendOobCode", self.base_url(), self.project_id);

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

         let response = self.client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

         if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!("Generate email link failed {}: {}", status, text)));
        }

        let result: EmailLinkResponse = response.json().await?;
        Ok(result.oob_link)
    }

    pub async fn generate_password_reset_link(&self, email: &str, settings: Option<serde_json::Value>) -> Result<String, AuthError> {
        self.generate_email_link("PASSWORD_RESET", email, settings).await
    }

    pub async fn generate_email_verification_link(&self, email: &str, settings: Option<serde_json::Value>) -> Result<String, AuthError> {
        self.generate_email_link("VERIFY_EMAIL", email, settings).await
    }

    pub async fn generate_sign_in_with_email_link(&self, email: &str, settings: Option<serde_json::Value>) -> Result<String, AuthError> {
        self.generate_email_link("EMAIL_SIGNIN", email, settings).await
    }

    pub async fn import_users(&self, request: ImportUsersRequest) -> Result<ImportUsersResponse, AuthError> {
        let url = format!("{}/{}/accounts:batchCreate", self.base_url(), self.project_id);

        let response = self.client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!("Import users failed {}: {}", status, text)));
        }

        let result: ImportUsersResponse = response.json().await?;

        if let Some(errors) = &result.error {
             if !errors.is_empty() {
                 // Partial failure or full failure reporting depending on API behavior
                 // Usually batchCreate returns 200 with errors list for partials.
                 // We can return the response or error out.
                 // Let's return the response but user should check it.
                 // Or we can define that if errors exist, we return Err(AuthError::ImportUsersError(errors))
                 return Err(AuthError::ImportUsersError(errors.iter().map(|e| crate::auth::models::ImportUserError {
                     index: e.index,
                     message: e.message.clone(),
                 }).collect()));
             }
        }

        Ok(result)
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserRecord, AuthError> {
        let url = format!("{}/{}/accounts", self.base_url(), self.project_id);

        let response = self.client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!("Create user failed {}: {}", status, text)));
        }

        let user: UserRecord = response.json().await?;
        Ok(user)
    }

    pub async fn update_user(&self, request: UpdateUserRequest) -> Result<UserRecord, AuthError> {
        let url = format!("{}/{}/accounts:update", self.base_url(), self.project_id);

        let response = self.client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!("Update user failed {}: {}", status, text)));
        }

        let user: UserRecord = response.json().await?;
        Ok(user)
    }

    pub async fn delete_user(&self, uid: &str) -> Result<(), AuthError> {
        let url = format!("{}/{}/accounts:delete", self.base_url(), self.project_id);
        let request = DeleteAccountRequest { local_id: uid.to_string() };

        let response = self.client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!("Delete user failed {}: {}", status, text)));
        }

        Ok(())
    }

    // Helper to get account info
    async fn get_account_info(&self, request: GetAccountInfoRequest) -> Result<UserRecord, AuthError> {
        let url = format!("{}/{}/accounts:lookup", self.base_url(), self.project_id);

        let response = self.client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!("Get user failed {}: {}", status, text)));
        }

        let result: GetAccountInfoResponse = response.json().await?;

        result.users
            .and_then(|mut users| users.pop())
            .ok_or(AuthError::UserNotFound)
    }

    pub async fn get_user(&self, uid: &str) -> Result<UserRecord, AuthError> {
        let request = GetAccountInfoRequest {
            local_id: Some(vec![uid.to_string()]),
            email: None,
            phone_number: None,
        };
        self.get_account_info(request).await
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<UserRecord, AuthError> {
        let request = GetAccountInfoRequest {
            local_id: None,
            email: Some(vec![email.to_string()]),
            phone_number: None,
        };
        self.get_account_info(request).await
    }

    pub async fn get_user_by_phone_number(&self, phone: &str) -> Result<UserRecord, AuthError> {
        let request = GetAccountInfoRequest {
            local_id: None,
            email: None,
            phone_number: Some(vec![phone.to_string()]),
        };
        self.get_account_info(request).await
    }

    pub async fn list_users(&self, max_results: u32, page_token: Option<&str>) -> Result<ListUsersResponse, AuthError> {
        let url = format!("{}/{}/accounts", self.base_url(), self.project_id);

        // Query params
        let mut params = Vec::new();
        params.push(("maxResults", max_results.to_string()));
        if let Some(token) = page_token {
            params.push(("nextPageToken", token.to_string()));
        }

        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::ApiError(format!("List users failed {}: {}", status, text)));
        }

        let result: ListUsersResponse = response.json().await?;
        Ok(result)
    }
}
