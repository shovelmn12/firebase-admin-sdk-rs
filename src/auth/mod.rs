pub mod models;

use reqwest_middleware::ClientWithMiddleware;
use crate::auth::models::{
    CreateUserRequest, DeleteAccountRequest, GetAccountInfoRequest, GetAccountInfoResponse,
    ListUsersResponse, UpdateUserRequest, UserRecord,
};
use thiserror::Error;
use reqwest::header;

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
}

#[derive(Clone)]
pub struct FirebaseAuth {
    client: ClientWithMiddleware,
    project_id: String,
}

impl FirebaseAuth {
    pub fn new(client: ClientWithMiddleware, project_id: String) -> Self {
        Self { client, project_id }
    }

    // Base URL for Identity Toolkit API
    fn base_url(&self) -> String {
        "https://identitytoolkit.googleapis.com/v1/projects".to_string()
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
