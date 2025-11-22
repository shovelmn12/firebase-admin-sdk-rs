use yup_oauth2::ServiceAccountKey;
use reqwest_middleware::ClientWithMiddleware;

use crate::core::http_client::create_client;

pub mod models;

pub struct FirebaseRemoteConfig {
    client: ClientWithMiddleware,
    base_url: String,
}

use self::models::{RemoteConfig, Version};

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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the service account key is missing the project_id")]
    ProjectIdMissing,
    #[error("an error occurred while sending the request: {0}")]
    Request(#[from] reqwest_middleware::Error),
    #[error("an error occurred while serializing/deserializing JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("the firebase API returned an error: {code} {status}: {message}")]
    Api {
        code: u16,
        message: String,
        status: String,
    },
}

impl FirebaseRemoteConfig {
    pub fn new(key: ServiceAccountKey) -> Result<Self, Error> {
        let project_id = key.project_id.clone().ok_or(Error::ProjectIdMissing)?;
        if project_id.is_empty() {
            return Err(Error::ProjectIdMissing);
        }

        let client = create_client(key);
        let base_url = REMOTE_CONFIG_V1_API.replace("{project_id}", &project_id);

        Ok(Self { client, base_url })
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

    pub async fn get(&self) -> Result<RemoteConfig, Error> {
        let response = self.client.get(&self.base_url).send().await?;
        self.process_response(response).await
    }

    pub async fn publish(&self, config: RemoteConfig) -> Result<RemoteConfig, Error> {
        let response = self
            .client
            .put(&self.base_url)
            .header("If-Match", config.etag.clone())
            .json(&config)
            .send()
            .await?;
        self.process_response(response).await
    }

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

    pub async fn rollback(&self, version_number: String) -> Result<RemoteConfig, Error> {
        let url = format!("{}:rollback", self.base_url);
        let body = models::RollbackRequest { version_number };

        let response = self.client.post(url).json(&body).send().await?;
        self.process_response(response).await
    }
}
