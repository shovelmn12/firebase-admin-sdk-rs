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
pub mod snapshot;
pub mod transaction;
pub mod batch;

#[cfg(test)]
mod tests;

use self::batch::WriteBatch;
use self::query::{ExecutableQuery, Query};
use self::reference::{CollectionReference, DocumentReference};
use self::transaction::Transaction;
use crate::core::middleware::AuthMiddleware;
use crate::core::parse_error_response;
use crate::firestore::models::{
    BeginTransactionRequest, BeginTransactionResponse, ListCollectionIdsRequest,
    ListCollectionIdsResponse, RollbackRequest, TransactionOptions,
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

    #[cfg(test)]
    pub(crate) fn new_with_client(client: ClientWithMiddleware, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Gets a `CollectionReference` instance that refers to the collection at the specified path.
    ///
    /// # Arguments
    ///
    /// * `collection_id` - The ID of the collection (e.g., "users").
    pub fn collection(&'_ self, collection_id: &str) -> CollectionReference<'_> {
        CollectionReference {
            client: &self.client,
            path: format!("{}/{}", self.base_url, collection_id),
        }
    }

    /// Lists the root collections of the database.
    pub async fn list_collections(&self) -> Result<Vec<CollectionReference<'_>>, FirestoreError> {
        let url = format!("{}:listCollectionIds", self.base_url);
        let mut collections = Vec::new();
        let mut next_page_token = None;

        loop {
            let request = ListCollectionIdsRequest {
                page_size: Some(100),
                page_token: next_page_token.take(),
            };

            let response = self
                .client
                .post(&url)
                .header(header::CONTENT_TYPE, "application/json")
                .body(serde_json::to_vec(&request)?)
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(FirestoreError::ApiError(parse_error_response(response, "List collections failed").await));
            }

            let result: ListCollectionIdsResponse = response.json().await?;
            for id in result.collection_ids {
                collections.push(self.collection(&id));
            }

            if let Some(token) = result.next_page_token {
                if token.is_empty() {
                    break;
                }
                next_page_token = Some(token);
            } else {
                break;
            }
        }

        Ok(collections)
    }

    /// Gets a `DocumentReference` instance that refers to the document at the specified path.
    ///
    /// # Arguments
    ///
    /// * `document_path` - The slash-separated path to the document (e.g., "users/user1").
    pub fn doc(&self, document_path: &str) -> DocumentReference {
        DocumentReference {
            client: &self.client,
            path: format!("{}/{}", self.base_url, document_path),
        }
    }

    /// Creates a write batch, used for performing multiple writes as a single atomic operation.
    pub fn batch(&self) -> WriteBatch<'_> {
        WriteBatch::new(&self.client, self.base_url.clone())
    }

    /// Creates an executable query from a query definition.
    ///
    /// # Arguments
    ///
    /// * `query` - The `Query` definition containing filters and the target collection.
    pub fn query(&self, query: Query) -> ExecutableQuery<'_> {
        ExecutableQuery::new(&self.client, self.base_url.clone(), query)
    }

    /// Begins a new transaction.
    ///
    /// This method is for manual transaction management. For automatic retries, use `run_transaction`.
    pub async fn begin_transaction(
        &self,
        options: Option<TransactionOptions>,
    ) -> Result<Transaction, FirestoreError> {
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
            return Err(FirestoreError::ApiError(parse_error_response(response, "Begin transaction failed").await));
        }

        let result: BeginTransactionResponse = response.json().await?;
        Ok(Transaction::new(
            self.client.clone(),
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
            return Err(FirestoreError::ApiError(parse_error_response(response, "Rollback transaction failed").await));
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
