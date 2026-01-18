//! Cloud Storage for Firebase module.
//!
//! This module provides functionality for interacting with Google Cloud Storage buckets
//! associated with your Firebase project. It supports uploading, downloading, and deleting files,
//! as well as managing file metadata.
//!
//! # Examples
//!
//! ```rust,ignore
//! # use firebase_admin_sdk::FirebaseApp;
//! # async fn run(app: FirebaseApp) {
//! let storage = app.storage();
//! let bucket = storage.bucket(None); // Use default bucket
//!
//! // Upload a file
//! let file_content = b"Hello, World!".to_vec();
//! let file = bucket.file("hello.txt");
//! let _ = file.save(file_content, "text/plain").await;
//! # }
//! ```

pub mod bucket;
pub mod file;

use crate::core::middleware::AuthMiddleware;
use bucket::Bucket;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use thiserror::Error;

const STORAGE_V1_API: &str = "https://storage.googleapis.com/storage/v1";

/// Errors that can occur during Storage operations.
#[derive(Error, Debug)]
pub enum StorageError {
    /// Wrapper for `reqwest::Error`.
    #[error("HTTP Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    /// Wrapper for `reqwest_middleware::Error`.
    #[error("Middleware error: {0}")]
    MiddlewareError(#[from] reqwest_middleware::Error),
    /// Errors returned by the Cloud Storage API.
    #[error("API error: {0}")]
    ApiError(String),
    /// Wrapper for `serde_json::Error`.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    /// Missing project ID in service account key.
    #[error("Project ID is missing in service account key")]
    ProjectIdMissing,
}

/// Client for interacting with Cloud Storage for Firebase.
#[derive(Clone)]
pub struct FirebaseStorage {
    client: ClientWithMiddleware,
    pub base_url: String,
    pub project_id: String,
    middleware: AuthMiddleware,
}

impl FirebaseStorage {
    /// Creates a new `FirebaseStorage` instance.
    ///
    /// This is typically called via `FirebaseApp::storage()`.
    pub fn new(middleware: AuthMiddleware) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(middleware.clone())
            .build();

        let project_id = middleware.key.project_id.clone().unwrap_or_default();
        let base_url = STORAGE_V1_API.to_string();

        Self {
            client,
            base_url,
            project_id,
            middleware,
        }
    }

    /// Gets a `Bucket` instance that refers to the specific bucket.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the bucket (e.g. "my-project.appspot.com").
    ///            If not provided, it attempts to use the default bucket name derived from the project ID
    ///            (e.g., "{project_id}.appspot.com").
    pub fn bucket(&self, name: Option<&str>) -> Bucket {
        let bucket_name = match name {
            Some(n) => n.to_string(),
            None => format!("{}.appspot.com", self.project_id),
        };

        Bucket::new(self.client.clone(), self.base_url.clone(), bucket_name, self.middleware.clone())
    }
}

#[cfg(test)]
mod tests;
