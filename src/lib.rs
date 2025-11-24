//! # Firebase Admin SDK for Rust
//!
//! This crate provides a Rust implementation of the Firebase Admin SDK, allowing interaction
//! with Firebase services such as Authentication, Cloud Messaging (FCM), Remote Config, and Firestore.
//!
//! ## Modules
//!
//! - [`auth`]: Firebase Authentication (User management, ID token verification).
//! - [`messaging`]: Firebase Cloud Messaging (Send messages, topic management).
//! - [`remote_config`]: Firebase Remote Config (Get, update, rollback configurations).
//! - [`firestore`]: Cloud Firestore (Read/Write documents).
//!
//! ## Usage
//!
//! The entry point is the [`FirebaseApp`] struct. You initialize it with a `ServiceAccountKey`
//! (typically loaded from a JSON file), and then access the various services.
//!
//! ```rust,no_run
//! use firebase_admin_sdk::{FirebaseApp, yup_oauth2};
//!
//! async fn example() {
//!     let key = yup_oauth2::read_service_account_key("service-account.json").await.unwrap();
//!
//!     let app = FirebaseApp::new(key);
//!     let auth = app.auth();
//!     let messaging = app.messaging();
//! }
//! ```

pub mod auth;
pub mod core;
pub mod firestore;
pub mod messaging;
pub mod remote_config;
pub mod storage;

// Re-export yup_oauth2 for user convenience so they don't need to add it separately
pub use yup_oauth2;

use auth::FirebaseAuth;
use core::middleware::AuthMiddleware;
use firestore::FirebaseFirestore;
use messaging::FirebaseMessaging;
use remote_config::FirebaseRemoteConfig;
use storage::FirebaseStorage;
use yup_oauth2::ServiceAccountKey;

/// The entry point for the Firebase Admin SDK.
///
/// `FirebaseApp` holds the service account credentials and acts as a factory for creating
/// clients for specific Firebase services (Auth, Messaging, etc.).
///
/// It uses a "synchronous constructor, lazy async authentication" pattern.
/// The `new` method is synchronous and cheap, while the actual OAuth2 authentication
/// happens asynchronously and lazily upon the first API request made by any service client.
pub struct FirebaseApp {
    middleware: AuthMiddleware,
}

impl FirebaseApp {
    /// Creates a new `FirebaseApp` instance.
    ///
    /// This method is synchronous. The service account key is stored, but no network
    /// requests are made until a service (like `auth()` or `messaging()`) actually performs an action.
    ///
    /// # Arguments
    ///
    /// * `service_account_key` - A `yup_oauth2::ServiceAccountKey` struct containing the credentials.
    pub fn new(service_account_key: ServiceAccountKey) -> Self {
        Self {
            middleware: AuthMiddleware::new(service_account_key),
        }
    }

    /// Returns a client for interacting with Firebase Authentication.
    pub fn auth(&self) -> FirebaseAuth {
        FirebaseAuth::new(self.middleware.clone())
    }

    /// Returns a client for interacting with Firebase Cloud Messaging (FCM).
    pub fn messaging(&self) -> FirebaseMessaging {
        FirebaseMessaging::new(self.middleware.clone())
    }

    /// Returns a client for interacting with Firebase Remote Config.
    pub fn remote_config(&self) -> FirebaseRemoteConfig {
        FirebaseRemoteConfig::new(self.middleware.clone())
    }

    /// Returns a client for interacting with Cloud Firestore.
    pub fn firestore(&self) -> FirebaseFirestore {
        FirebaseFirestore::new(self.middleware.clone())
    }

    /// Returns a client for interacting with Firebase Storage.
    pub fn storage(&self) -> FirebaseStorage {
        FirebaseStorage::new(self.middleware.clone())
    }
}
