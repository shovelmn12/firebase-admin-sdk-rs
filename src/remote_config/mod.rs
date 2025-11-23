//! Firebase Remote Config module.
//!
//! This module provides functionality to read and modify the Remote Config template.
//!
//! # Optimistic Concurrency
//!
//! The `publish` method uses the ETag from the fetched configuration to ensure optimistic concurrency.
//! If the remote configuration has changed since it was fetched, the publish operation will fail.

pub mod models;

use crate::core::middleware::AuthMiddleware;
use crate::remote_config::models::RemoteConfig;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use yup_oauth2::ServiceAccountKey;

/// Client for interacting with Firebase Remote Config.
pub struct FirebaseRemoteConfig {
    client: ClientWithMiddleware,
    base_url: String,
}

const REMOTE_CONFIG_V1_API: &str =
    "https://firebaseremoteconfig.googleapis.com/v1/projects/{project_id}/remoteConfig";

#[derive(Debug, serde::Deserialize)]
struct ApiError {
    code: u16,
    message: String,
    status: String,
}

#[derive(Debug, serde::Deserialize)]
struct ErrorWrapper {
    error: ApiError,
}

/// Errors that can occur during Remote Config operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The service account key provided does not contain a project ID.
    #[error("the service account key is missing the project_id")]
    ProjectIdMissing,
    /// Wrapper for `reqwest_middleware::Error`.
    #[error("an error occurred while sending the request: {0}")]
    Request(#[from] reqwest_middleware::Error),
    /// Wrapper for `reqwest::Error`.
    #[error("an error occurred while sending the request: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Wrapper for `serde_json::Error`.
    #[error("an error occurred while serializing/deserializing JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Error returned by the Remote Config API.
    #[error("the firebase API returned an error: {code} {status}: {message}")]
    Api {
        code: u16,
        message: String,
        status: String,
    },
}

impl FirebaseRemoteConfig {
    /// Creates a new `FirebaseRemoteConfig` instance.
    ///
    /// This is typically called via `FirebaseApp::remote_config()`.
    pub fn new(key: ServiceAccountKey) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(AuthMiddleware::new(key.clone()))
            .build();

        let project_id = key.project_id.unwrap_or_default();
        let base_url = REMOTE_CONFIG_V1_API.replace("{project_id}", &project_id);

        Self { client, base_url }
    }

    async fn process_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, Error> {
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: ErrorWrapper = response.json().await?;
            Err(Error::Api {
                code: error.error.code,
                message: error.error.message,
                status: error.error.status,
            })
        }
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<(T, Option<String>), Error> {
        let response = req.send().await?;
        if !response.status().is_success() {
            let error: ErrorWrapper = response.json().await?;
            return Err(Error::Api {
                code: error.error.code,
                message: error.error.message,
                status: error.error.status,
            });
        }
        let etag = response
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let body: T = response.json().await?;
        Ok((body, etag))
    }

    /// Fetches the current active Remote Config template.
    ///
    /// The returned `RemoteConfig` object contains an ETag which is used for optimistic locking during updates.
    pub async fn get(&self) -> Result<RemoteConfig, Error> {
        let req = self.client.get(&self.base_url);
        let (mut config, etag) = self.request::<RemoteConfig>(req).await?;
        if let Some(e) = etag {
            config.etag = e;
        }
        Ok(config)
    }

    /// Publishes a new Remote Config template.
    ///
    /// This method includes the `If-Match` header using the ETag present in the `config` object.
    /// If the ETag does not match the server's current version, the request will fail.
    ///
    /// # Arguments
    ///
    /// * `config` - The `RemoteConfig` template to publish.
    pub async fn publish(&self, config: RemoteConfig) -> Result<RemoteConfig, Error> {
        let req = self
            .client
            .put(&self.base_url)
            .header("If-Match", config.etag.clone())
            .json(&config);
        let (mut config, etag) = self.request::<RemoteConfig>(req).await?;
        if let Some(e) = etag {
            config.etag = e;
        }
        Ok(config)
    }

    /// Lists previous versions of the Remote Config template.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional query parameters for pagination and filtering.
    pub async fn list_versions(
        &self,
        options: Option<models::ListVersionsOptions>,
    ) -> Result<models::ListVersionsResult, Error> {
        let url = format!("{}/versions", self.base_url);
        let response = self
            .client
            .get(url)
            .query(&options.unwrap_or_default())
            .send()
            .await?;
        self.process_response(response).await
    }

    /// Rolls back the Remote Config template to a specific version.
    ///
    /// # Arguments
    ///
    /// * `version_number` - The version number to roll back to.
    pub async fn rollback(&self, version_number: String) -> Result<RemoteConfig, Error> {
        let url = format!("{}:rollback", self.base_url);
        let body = models::RollbackRequest { version_number };

        let req = self.client.post(url).json(&body);
        let (mut config, etag) = self.request::<RemoteConfig>(req).await?;
        if let Some(e) = etag {
            config.etag = e;
        }
        Ok(config)
    }
}
