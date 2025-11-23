pub mod models;
pub mod reference;

use self::reference::{CollectionReference, DocumentReference};
use crate::core::middleware::AuthMiddleware;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use thiserror::Error;

const FIRESTORE_V1_API: &str =
    "https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents";

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
    base_url: String,
}

impl FirebaseFirestore {
    pub fn new(middleware: AuthMiddleware) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(middleware.clone())
            .build();

        let project_id = middleware.key().project_id.clone().unwrap_or_default();
        let base_url = FIRESTORE_V1_API.replace("{project_id}", &project_id);

        Self { client, base_url }
    }

    pub fn collection<'a>(&'a self, collection_id: &str) -> CollectionReference<'a> {
        CollectionReference {
            client: &self.client,
            path: format!("{}/{}", self.base_url, collection_id),
        }
    }

    pub fn doc<'a>(&'a self, document_path: &str) -> DocumentReference<'a> {
        DocumentReference {
            client: &self.client,
            path: format!("{}/{}", self.base_url, document_path),
        }
    }
}

#[cfg(test)]
mod tests;
