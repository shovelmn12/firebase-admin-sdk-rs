use reqwest::{Client, header};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use crate::core::middleware::AuthMiddleware;
use crate::messaging::models::Message;
use thiserror::Error;
use yup_oauth2::ServiceAccountKey;
use serde::{Deserialize, Serialize};

pub mod models;
#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
pub enum MessagingError {
    #[error("HTTP Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Middleware error: {0}")]
    MiddlewareError(#[from] reqwest_middleware::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct FirebaseMessaging {
    client: ClientWithMiddleware,
    project_id: String,
}

#[derive(Deserialize)]
struct SendResponse {
    name: String,
}

// Wrapper for the request body required by FCM v1 API
#[derive(Serialize)]
struct SendRequest<'a> {
    validate_only: bool,
    message: &'a Message,
}

impl FirebaseMessaging {
    pub fn new(service_account_key: ServiceAccountKey) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(AuthMiddleware::new(service_account_key.clone()))
            .build();

        let project_id = service_account_key.project_id.unwrap_or_default();

        Self {
            client,
            project_id,
        }
    }

    pub async fn send(&self, message: &Message) -> Result<String, MessagingError> {
        self.send_request(message, false).await
    }

    pub async fn send_dry_run(&self, message: &Message) -> Result<String, MessagingError> {
        self.send_request(message, true).await
    }

    async fn send_request(&self, message: &Message, dry_run: bool) -> Result<String, MessagingError> {
        let url = format!("https://fcm.googleapis.com/v1/projects/{}/messages:send", self.project_id);

        let request = SendRequest {
            validate_only: dry_run,
            message,
        };

        let response = self.client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(MessagingError::ApiError(format!("FCM send failed {}: {}", status, text)));
        }

        let result: SendResponse = response.json().await?;
        Ok(result.name)
    }
}
