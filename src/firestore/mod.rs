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

use self::reference::{CollectionReference, DocumentReference};
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
}
