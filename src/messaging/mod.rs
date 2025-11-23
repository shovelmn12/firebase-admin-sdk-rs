use reqwest::{Client, header};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use crate::core::middleware::AuthMiddleware;
use crate::messaging::models::{Message, TopicManagementResponse, TopicManagementError, BatchResponse, SendResponse};
use thiserror::Error;
use yup_oauth2::ServiceAccountKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use futures::{StreamExt, stream};

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
struct SendResponseInternal {
    name: String,
}

// Wrapper for the request body required by FCM v1 API
#[derive(Serialize)]
struct SendRequest<'a> {
    validate_only: bool,
    message: &'a Message,
}

#[derive(Serialize)]
struct TopicManagementRequest<'a> {
    to: String,
    registration_tokens: &'a [&'a str],
}

#[derive(Deserialize)]
struct TopicManagementApiResponse {
    results: Option<Vec<TopicManagementApiResult>>,
}

#[derive(Deserialize)]
struct TopicManagementApiResult {
    error: Option<String>,
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

        let result: SendResponseInternal = response.json().await?;
        Ok(result.name)
    }

    pub async fn send_each(&self, messages: &[Message]) -> Result<BatchResponse, MessagingError> {
        self.send_each_request(messages, false).await
    }

    pub async fn send_each_dry_run(&self, messages: &[Message]) -> Result<BatchResponse, MessagingError> {
        self.send_each_request(messages, true).await
    }

    async fn send_each_request(&self, messages: &[Message], dry_run: bool) -> Result<BatchResponse, MessagingError> {
        // Concurrency limit to prevent overwhelming the client or server
        const CONCURRENCY_LIMIT: usize = 50;

        let responses = stream::iter(messages)
            .map(|message| {
                let client = self.clone();
                async move {
                    match client.send_request(message, dry_run).await {
                        Ok(id) => SendResponse {
                            success: true,
                            message_id: Some(id),
                            error: None,
                        },
                        Err(e) => SendResponse {
                            success: false,
                            message_id: None,
                            error: Some(e.to_string()),
                        },
                    }
                }
            })
            // Use buffered instead of buffer_unordered to ensure the output order matches input order.
            .buffered(CONCURRENCY_LIMIT)
            .collect::<Vec<SendResponse>>()
            .await;

        let success_count = responses.iter().filter(|r| r.success).count();
        let failure_count = responses.len() - success_count;

        Ok(BatchResponse {
            success_count,
            failure_count,
            responses,
        })
    }

    pub async fn send_multicast(&self, message: &Message, tokens: &[&str]) -> Result<BatchResponse, MessagingError> {
        self.send_multicast_request(message, tokens, false).await
    }

    pub async fn send_multicast_dry_run(&self, message: &Message, tokens: &[&str]) -> Result<BatchResponse, MessagingError> {
        self.send_multicast_request(message, tokens, true).await
    }

    async fn send_multicast_request(&self, base_message: &Message, tokens: &[&str], dry_run: bool) -> Result<BatchResponse, MessagingError> {
        let messages: Vec<Message> = tokens.iter().map(|token| {
            let mut msg = base_message.clone();
            msg.token = Some(token.to_string());
            msg
        }).collect();

        self.send_each_request(&messages, dry_run).await
    }

    pub async fn subscribe_to_topic(&self, topic: &str, tokens: &[&str]) -> Result<TopicManagementResponse, MessagingError> {
        self.manage_topic(topic, tokens, true).await
    }

    pub async fn unsubscribe_from_topic(&self, topic: &str, tokens: &[&str]) -> Result<TopicManagementResponse, MessagingError> {
        self.manage_topic(topic, tokens, false).await
    }

    async fn manage_topic(&self, topic: &str, tokens: &[&str], subscribe: bool) -> Result<TopicManagementResponse, MessagingError> {
        let topic_path = if topic.starts_with("/topics/") {
            topic.to_string()
        } else {
            format!("/topics/{}", topic)
        };

        let url = if subscribe {
            "https://iid.googleapis.com/iid/v1:batchAdd"
        } else {
            "https://iid.googleapis.com/iid/v1:batchRemove"
        };

        let mut response_summary = TopicManagementResponse::default();

        for (batch_idx, chunk) in tokens.chunks(1000).enumerate() {
            let request = TopicManagementRequest {
                to: topic_path.clone(),
                registration_tokens: chunk,
            };

            let response = self.client
                .post(url)
                .header(header::CONTENT_TYPE, "application/json")
                // Use access_token_header from AuthMiddleware, but the IID API also requires the standard header.
                // The AuthMiddleware adds it automatically.
                .header("access_token_auth", "true") // Some docs suggest this for IID, but standard Bearer should work.
                .body(serde_json::to_vec(&request)?)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(MessagingError::ApiError(format!("Topic management failed {}: {}", status, text)));
            }

            let api_response: TopicManagementApiResponse = response.json().await?;

            if let Some(results) = api_response.results {
                for (i, result) in results.iter().enumerate() {
                     if let Some(error) = &result.error {
                         response_summary.failure_count += 1;
                         response_summary.errors.push(TopicManagementError {
                             index: batch_idx * 1000 + i,
                             reason: error.clone(),
                         });
                     } else {
                         response_summary.success_count += 1;
                     }
                }
            }
        }

        Ok(response_summary)
    }
}
