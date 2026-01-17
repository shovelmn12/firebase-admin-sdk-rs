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
pub mod reference;
pub mod transaction;

// #[cfg(test)]
// mod tests;

use self::reference::{CollectionReference, DocumentReference};
use self::transaction::Transaction;
use crate::core::middleware::AuthMiddleware;
use crate::firestore::models::{
    BeginTransactionRequest, BeginTransactionResponse, RollbackRequest, TransactionOptions,
};
use reqwest::{header, Client};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::future::Future;
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
    /// Transaction was aborted (too many retries or explicit abort).
    #[error("Transaction failed: {0}")]
    TransactionError(String),
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

    /// Creates a new `FirebaseFirestore` instance with a custom base URL (useful for testing).
    pub fn new_with_url(middleware: AuthMiddleware, base_url: String) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(middleware.clone())
            .build();

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

    /// Begins a new transaction.
    ///
    /// This method is for manual transaction management. For automatic retries, use `run_transaction`.
    pub async fn begin_transaction(
        &self,
        options: Option<TransactionOptions>,
    ) -> Result<Transaction<'_>, FirestoreError> {
        let url = format!(
            "{}:beginTransaction",
            self.base_url.split("/documents").next().unwrap()
        );

        let request = BeginTransactionRequest { options };

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
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

    /// Rolls back a transaction.
    pub async fn rollback(&self, transaction_id: &str) -> Result<(), FirestoreError> {
        let url = format!(
            "{}:rollback",
            self.base_url.split("/documents").next().unwrap()
        );

        let request = RollbackRequest {
            transaction: transaction_id.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Rollback transaction failed {}: {}",
                status, text
            )));
        }

        Ok(())
    }

    /// Runs the given update function within a transaction.
    ///
    /// The update function may be called multiple times if the transaction is aborted due to contention.
    ///
    /// # Arguments
    ///
    /// * `update_fn` - A closure that takes a `Transaction` and returns a `Future`.
    pub async fn run_transaction<F, Fut, R>(&self, update_fn: F) -> Result<R, FirestoreError>
    where
        F: Fn(Transaction) -> Fut,
        Fut: Future<Output = Result<R, FirestoreError>>,
    {
        let mut retry_count = 0;
        let max_retries = 5;

        loop {
            let transaction = self.begin_transaction(None).await?;
            let transaction_id = transaction.transaction_id.clone();

            // Clone transaction to pass to update_fn, keeping one copy to commit
            let transaction_clone = transaction.clone();

            match update_fn(transaction_clone).await {
                Ok(result) => {
                    match transaction.commit().await {
                         Ok(_) => return Ok(result),
                         Err(FirestoreError::ApiError(msg)) if msg.contains("ABORTED") || msg.contains("status: 409") || msg.contains("Aborted") => {
                             // Check for contention (status ABORTED or 409)
                             retry_count += 1;
                             if retry_count >= max_retries {
                                 return Err(FirestoreError::TransactionError("Max retries reached".into()));
                             }
                             // Exponential backoff could be added here
                             continue;
                         }
                         Err(e) => return Err(e),
                    }
                }
                Err(e) => {
                     let _ = self.rollback(&transaction_id).await;
                     return Err(e);
                }
            }
        }
    }
}
