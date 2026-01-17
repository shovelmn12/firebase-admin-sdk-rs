//! Cloud Firestore module.
//!
//! This module provides functionality for interacting with Cloud Firestore,
//! including getting references to collections and documents, and listening for real-time updates.
//!
//! It mirrors the Firebase Admin Node.js SDK's structure using `CollectionReference` and `DocumentReference`.
//!
//! # Real-time Updates
//!
//! You can listen for changes to a document or an entire collection using the `listen()` method
//! on `DocumentReference` and `CollectionReference`. This returns a stream of `ListenResponse` events.

pub mod listen;
pub mod models;
pub mod query;
pub mod reference;
pub mod transaction;

use self::models::{BeginTransactionRequest, BeginTransactionResponse};
use self::reference::{CollectionReference, DocumentReference};
use self::transaction::Transaction;
pub use self::query::Query;
use crate::core::middleware::AuthMiddleware;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use thiserror::Error;

const FIRESTORE_V1_API: &str =
    "https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents";

/// Errors that can occur during Firestore operations.
#[derive(Error, Debug)]
pub enum FirestoreError {
    /// Wrapper for `reqwest::Error`.
    #[error("HTTP Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    /// Wrapper for `reqwest_middleware::Error`.
    #[error("Middleware error: {0}")]
    MiddlewareError(#[from] reqwest_middleware::Error),
    /// Errors returned by the Firestore API.
    #[error("API error: {0}")]
    ApiError(String),
    /// Wrapper for `serde_json::Error`.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Client for interacting with Cloud Firestore.
pub struct FirebaseFirestore {
    client: ClientWithMiddleware,
    base_url: String,
}

impl FirebaseFirestore {
    /// Creates a new `FirebaseFirestore` instance.
    ///
    /// This is typically called via `FirebaseApp::firestore()`.
    pub fn new(middleware: AuthMiddleware) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(middleware.clone())
            .build();

        let project_id = middleware.key.project_id.clone().unwrap_or_default();
        let base_url = FIRESTORE_V1_API.replace("{project_id}", &project_id);

        Self { client, base_url }
    }

    /// Gets a `CollectionReference` instance that refers to the collection at the specified path.
    ///
    /// # Arguments
    ///
    /// * `collection_id` - The ID of the collection (e.g., "users").
    pub fn collection<'a>(&'a self, collection_id: &str) -> CollectionReference<'a> {
        CollectionReference {
            client: &self.client,
            path: format!("{}/{}", self.base_url, collection_id),
        }
    }

    /// Gets a `DocumentReference` instance that refers to the document at the specified path.
    ///
    /// # Arguments
    ///
    /// * `document_path` - The slash-separated path to the document (e.g., "users/user1").
    pub fn doc<'a>(&'a self, document_path: &str) -> DocumentReference<'a> {
        DocumentReference {
            client: &self.client,
            path: format!("{}/{}", self.base_url, document_path),
        }
    }

    /// Starts a new transaction.
    ///
    /// # Returns
    ///
    /// A `Transaction` object that can be used to read and write documents atomically.
    pub async fn begin_transaction(&self) -> Result<Transaction<'_>, FirestoreError> {
        let url = format!("{}:beginTransaction", self.base_url);

        let request = BeginTransactionRequest {
            options: None, // Default to ReadWrite
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Begin transaction failed {}: {}",
                status, text
            )));
        }

        let result: BeginTransactionResponse = response.json().await?;

        Ok(Transaction::new(
            &self.client,
            self.base_url.clone(),
            result.transaction,
        ))
    }
}
