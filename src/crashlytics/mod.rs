//! Firebase Crashlytics module.
//!
//! This module provides functionality for managing Crashlytics data.
//! Currently, it supports deleting crash reports for a specific user, which is useful for
//! privacy compliance (e.g., "Right to be Forgotten").
//!
//! # Examples
//!
//! ```rust,no_run
//! # use firebase_admin_sdk::FirebaseApp;
//! # async fn run(app: FirebaseApp) {
//! let crashlytics = app.crashlytics();
//!
//! // Delete crash reports for a user
//! let _ = crashlytics.delete_crash_reports("your-app-id", "user-uid").await;
//! # }
//! ```

use crate::core::middleware::AuthMiddleware;
use reqwest::Client;
use reqwest::StatusCode;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use thiserror::Error;

/// Error type for Firebase Crashlytics operations.
#[derive(Debug, Error)]
pub enum Error {
    /// An error occurred while sending the request or receiving the response.
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// The middleware encountered an error (e.g., authentication failed).
    #[error("Middleware error: {0}")]
    Middleware(#[from] reqwest_middleware::Error),
    /// The API returned an error status code.
    #[error("API error: {0}")]
    Api(StatusCode),
}

const CRASHLYTICS_V1_API: &str =
    "https://firebasecrashlytics.googleapis.com/v1alpha/projects/{project_id}";

/// Client for interacting with the Firebase Crashlytics API.
pub struct FirebaseCrashlytics {
    client: ClientWithMiddleware,
    base_url: String,
}

impl FirebaseCrashlytics {
    /// Creates a new `FirebaseCrashlytics` client.
    pub fn new(middleware: AuthMiddleware) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(middleware.clone())
            .build();

        let project_id = middleware
            .key
            .project_id
            .clone()
            .unwrap_or_default();

        let base_url = CRASHLYTICS_V1_API.replace("{project_id}", &project_id);

        Self { client, base_url }
    }

    /// Creates a new `FirebaseCrashlytics` client with a custom client and base URL.
    /// Internal use only, primarily for testing.
    #[allow(dead_code)]
    pub(crate) fn new_with_client(client: ClientWithMiddleware, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Enqueues a request to permanently remove crash reports associated with the specified user.
    ///
    /// # Arguments
    ///
    /// * `app_id` - The App ID (e.g., the Google App ID, like `1:1234567890:android:321abc456def7890`).
    /// * `user_id` - The unique identifier of the user whose crash reports should be deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or if the API returns a non-success status code.
    pub async fn delete_crash_reports(&self, app_id: &str, user_id: &str) -> Result<(), Error> {
        // The resource name format is: projects/{project}/apps/{app}/users/{user}/crashReports
        let url = format!(
            "{}/apps/{}/users/{}/crashReports",
            self.base_url, app_id, user_id
        );

        let response = self.client.delete(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::Api(response.status()))
        }
    }
}

#[cfg(test)]
mod tests;
