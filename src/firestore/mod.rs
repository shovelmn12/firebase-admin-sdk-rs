pub mod models;
pub mod reference;

use self::reference::{CollectionReference, DocumentReference};
use crate::core::middleware::AuthMiddleware;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use thiserror::Error;
use yup_oauth2::ServiceAccountKey;

#[derive(Error, Debug)]
pub enum FirestoreError {
    #[error("HTTP Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Middleware error: {0}")]
    MiddlewareError(#[from] reqwest_middleware::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub struct FirebaseFirestore {
    client: ClientWithMiddleware,
    project_id: String,
}

impl FirebaseFirestore {
    pub fn new(service_account_key: ServiceAccountKey) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(AuthMiddleware::new(service_account_key.clone()))
            .build();

        let project_id = service_account_key.project_id.clone().unwrap_or_default();

        Self {
            client,
            project_id,
        }
    }

    fn base_url(&self) -> String {
        format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents",
            self.project_id
        )
    }

    pub fn collection<'a>(&'a self, collection_id: &str) -> CollectionReference<'a> {
        CollectionReference {
            client: &self.client,
            path: format!("{}/{}", self.base_url(), collection_id),
        }
    }

    pub fn doc<'a>(&'a self, document_path: &str) -> DocumentReference<'a> {
        DocumentReference {
            client: &self.client,
            path: format!("{}/{}", self.base_url(), document_path),
        }
    }
}

#[cfg(test)]
mod tests;
