//! Firebase Cloud Messaging (FCM) module.
//!
//! This module provides functionality for sending messages via FCM (single, batch, multicast)
//! and managing topic subscriptions.
//!
//! # Examples
//!
//! ```rust,ignore
//! use firebase_admin_sdk::messaging::models::{Message, Notification};
//! # use firebase_admin_sdk::FirebaseApp;
//! # async fn run(app: FirebaseApp) {
//! let messaging = app.messaging();
//!
//! let message = Message {
//!     token: Some("device_token".to_string()),
//!     notification: Some(Notification {
//!         title: Some("Title".to_string()),
//!         body: Some("Body".to_string()),
//!         ..Default::default()
//!     }),
//!     ..Default::default()
//! };
//!
//! let result = messaging.send(&message, false).await;
//! # }
//! ```

use reqwest::{Client, header};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use crate::core::middleware::AuthMiddleware;
use crate::core::parse_error_response;
use crate::messaging::models::{Message, MulticastMessage, TopicManagementResponse, TopicManagementError, BatchResponse, SendResponse, SendResponseInternal};
use thiserror::Error;
use serde::{Deserialize, Serialize};

pub mod models;

#[cfg(test)]
mod tests;

/// Errors that can occur during Messaging operations.
#[derive(Error, Debug)]
pub enum MessagingError {
    /// Wrapper for `reqwest::Error`.
    #[error("HTTP Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    /// Wrapper for `reqwest_middleware::Error`.
    #[error("Middleware error: {0}")]
    MiddlewareError(#[from] reqwest_middleware::Error),
    /// Errors returned by the FCM API.
    #[error("API error: {0}")]
    ApiError(String),
    /// Wrapper for `serde_json::Error`.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    /// Error parsing multipart responses for batch requests.
    #[error("Multipart response parsing error: {0}")]
    MultipartError(String),
}

/// Client for interacting with Firebase Cloud Messaging.
#[derive(Clone)]
pub struct FirebaseMessaging {
    client: ClientWithMiddleware,
    project_id: String,
    base_url: String,
}

// Wrapper for the request body required by FCM v1 API
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
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
    /// Creates a new `FirebaseMessaging` instance.
    ///
    /// This is typically called via `FirebaseApp::messaging()`.
    pub fn new(middleware: AuthMiddleware) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(middleware.clone())
            .build();

        let project_id = middleware.key.project_id.clone().unwrap_or_default();
        let base_url = format!("https://fcm.googleapis.com/v1/projects/{}/messages:send", project_id);

        Self {
            client,
            project_id,
            base_url,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_url(middleware: AuthMiddleware, base_url: String) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(middleware.clone())
            .build();
        let project_id = middleware.key.project_id.clone().unwrap_or_default();
        Self { client, project_id, base_url }
    }

    /// Sends a message to a specific target (token, topic, or condition).
    ///
    /// # Arguments
    ///
    /// * `message` - The `Message` struct defining the payload and target.
    /// * `dry_run` - If true, the message will be validated but not sent.
    pub async fn send(&self, message: &Message, dry_run: bool) -> Result<String, MessagingError> {
        self.validate_message(message)?;
        self.send_request(message, dry_run).await
    }

    /// Validates that the message has exactly one target.
    fn validate_message(&self, message: &Message) -> Result<(), MessagingError> {
        let num_targets = [
            message.token.is_some(),
            message.topic.is_some(),
            message.condition.is_some(),
        ]
        .iter()
        .filter(|&&t| t)
        .count();

        if num_targets != 1 {
            return Err(MessagingError::ApiError(
                "Message must have exactly one of token, topic, or condition.".to_string(),
            ));
        }

        Ok(())
    }

    /// Internal method to send the HTTP request.
    async fn send_request(&self, message: &Message, dry_run: bool) -> Result<String, MessagingError> {
        let request = SendRequest {
            validate_only: dry_run,
            message,
        };

        let response = self.client
            .post(&self.base_url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MessagingError::ApiError(parse_error_response(response, "FCM send failed").await));
        }

        let result: SendResponseInternal = response.json().await?;
        Ok(result.name)
    }

    /// Sends a batch of messages.
    ///
    /// This uses the FCM batch endpoint to send up to 500 messages in a single HTTP request.
    ///
    /// # Arguments
    ///
    /// * `messages` - A slice of `Message` structs.
    /// * `dry_run` - If true, the messages will be validated but not sent.
    pub async fn send_each(&self, messages: &[Message], dry_run: bool) -> Result<BatchResponse, MessagingError> {
        for message in messages {
            self.validate_message(message)?;
        }
        self.send_each_request(messages, dry_run).await
    }

    async fn send_each_request(&self, messages: &[Message], dry_run: bool) -> Result<BatchResponse, MessagingError> {
        if messages.is_empty() {
            return Ok(BatchResponse::default());
        }

        if messages.len() > 500 {
            return Err(MessagingError::ApiError("Cannot send more than 500 messages in a single batch.".to_string()));
        }

        let url = format!("https://fcm.googleapis.com/batch");
        let boundary = format!("batch_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

        let body = self.build_multipart_body(messages, dry_run, &boundary)?;

        let content_type = format!("multipart/mixed; boundary={}", boundary);

        let response = self.client
            .post(&url)
            .header(header::CONTENT_TYPE, content_type)
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MessagingError::ApiError(parse_error_response(response, "FCM batch send failed").await));
        }

        let multipart_boundary = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .and_then(|ct| ct.split("boundary=").nth(1))
            .map(|s| s.to_string())
            .ok_or_else(|| MessagingError::MultipartError("Multipart boundary not found in response".to_string()))?;

        let text = response.text().await?;
        let responses = self.parse_multipart_response(&text, &multipart_boundary)?;

        let success_count = responses.iter().filter(|r| r.success).count();
        let failure_count = responses.len() - success_count;

        Ok(BatchResponse {
            success_count,
            failure_count,
            responses,
        })
    }

    fn build_multipart_body(&self, messages: &[Message], dry_run: bool, boundary: &str) -> Result<Vec<u8>, MessagingError> {
        let mut body = Vec::new();

        for message in messages {
            let send_request = SendRequest {
                validate_only: dry_run,
                message,
            };

            let post_url = format!("/v1/projects/{}/messages:send", self.project_id);
            let request_body = serde_json::to_string(&send_request)?;

            body.extend_from_slice(b"--");
            body.extend_from_slice(boundary.as_bytes());
            body.extend_from_slice(b"\r\n");
            body.extend_from_slice(b"Content-Type: application/http\r\n");
            body.extend_from_slice(b"Content-Transfer-Encoding: binary\r\n\r\n");
            body.extend_from_slice(b"POST ");
            body.extend_from_slice(post_url.as_bytes());
            body.extend_from_slice(b"\r\n");
            body.extend_from_slice(b"Content-Type: application/json\r\n");
            body.extend_from_slice(b"\r\n");
            body.extend_from_slice(request_body.as_bytes());
            body.extend_from_slice(b"\r\n");
        }

        body.extend_from_slice(b"--");
        body.extend_from_slice(boundary.as_bytes());
        body.extend_from_slice(b"--\r\n");

        Ok(body)
    }

    fn parse_multipart_response(&self, body: &str, boundary: &str) -> Result<Vec<SendResponse>, MessagingError> {
        let boundary = format!("--{}", boundary);
        let parts: Vec<&str> = body.split(&boundary)
            .filter(|p| !p.trim().is_empty() && p.trim() != "--")
            .collect();
        let mut responses = Vec::new();

        for part in parts {
            let http_part = part.trim();

            if let Some(inner_response_start) = http_part.find("\r\n\r\n") {
                let inner_response = &http_part[inner_response_start + 4..];

                if let Some(json_start) = inner_response.find("\r\n\r\n") {
                    let json_body = inner_response[json_start + 4..].trim();

                    if json_body.is_empty() {
                        return Err(MessagingError::MultipartError("Empty JSON body in response part".to_string()));
                    }

                    let status_line = inner_response.lines().next().unwrap_or("");
                    if status_line.contains("200 OK") {
                        match serde_json::from_str::<SendResponseInternal>(json_body) {
                            Ok(send_response) => responses.push(SendResponse {
                                success: true,
                                message_id: Some(send_response.name),
                                error: None,
                            }),
                            Err(_) => return Err(MessagingError::MultipartError("Failed to parse successful response part".to_string())),
                        }
                    } else { // It's an error response
                         match serde_json::from_str::<serde_json::Value>(json_body) {
                            Ok(error_response) => responses.push(SendResponse {
                                success: false,
                                message_id: None,
                                error: Some(error_response.to_string()),
                            }),
                            Err(_) => return Err(MessagingError::MultipartError("Failed to parse error response part".to_string())),
                        }
                    }
                } else {
                     return Err(MessagingError::MultipartError("Invalid inner HTTP response format".to_string()));
                }
            } else {
                return Err(MessagingError::MultipartError("Invalid multipart part format".to_string()));
            }
        }

        Ok(responses)
    }

    /// Sends a multicast message to all specified tokens.
    ///
    /// This is a wrapper around `send_each` that constructs individual messages for each token.
    ///
    /// # Arguments
    ///
    /// * `message` - The `MulticastMessage` containing tokens and payload.
    /// * `dry_run` - If true, the messages will be validated but not sent.
    pub async fn send_each_for_multicast(&self, message: &MulticastMessage, dry_run: bool) -> Result<BatchResponse, MessagingError> {
        let messages: Vec<Message> = message.tokens.iter().map(|token| {
            Message {
                token: Some(token.clone()),
                data: message.data.clone(),
                notification: message.notification.clone(),
                android: message.android.clone(),
                webpush: message.webpush.clone(),
                apns: message.apns.clone(),
                fcm_options: message.fcm_options.clone(),
                ..Default::default()
            }
        }).collect();

        self.send_each(&messages, dry_run).await
    }

    /// Subscribes a list of tokens to a topic.
    ///
    /// # Arguments
    ///
    /// * `tokens` - A list of device registration tokens.
    /// * `topic` - The name of the topic.
    pub async fn subscribe_to_topic(&self, tokens: &[&str], topic: &str) -> Result<TopicManagementResponse, MessagingError> {
        self.manage_topic(topic, tokens, true).await
    }

    /// Unsubscribes a list of tokens from a topic.
    ///
    /// # Arguments
    ///
    /// * `tokens` - A list of device registration tokens.
    /// * `topic` - The name of the topic.
    pub async fn unsubscribe_from_topic(&self, tokens: &[&str], topic: &str) -> Result<TopicManagementResponse, MessagingError> {
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
                return Err(MessagingError::ApiError(parse_error_response(response, "Topic management failed").await));
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
