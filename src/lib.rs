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
//! - [`storage`]: Cloud Storage (Upload, download, delete files).
//! - [`crashlytics`]: Firebase Crashlytics (Manage crash reports).
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

#[cfg(feature = "auth")]
pub mod auth;
pub mod core;
#[cfg(feature = "crashlytics")]
pub mod crashlytics;
#[cfg(feature = "firestore")]
pub mod firestore;
#[cfg(feature = "messaging")]
pub mod messaging;
#[cfg(feature = "remote_config")]
pub mod remote_config;
#[cfg(feature = "storage")]
pub mod storage;

// Re-export yup_oauth2 for user convenience so they don't need to add it separately
pub use yup_oauth2;

#[cfg(feature = "auth")]
use auth::FirebaseAuth;
use core::middleware::AuthMiddleware;
#[cfg(feature = "crashlytics")]
use crashlytics::FirebaseCrashlytics;
#[cfg(feature = "firestore")]
use firestore::FirebaseFirestore;
#[cfg(feature = "messaging")]
use messaging::FirebaseMessaging;
#[cfg(feature = "remote_config")]
use remote_config::FirebaseRemoteConfig;
#[cfg(feature = "storage")]
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
    #[cfg(feature = "auth")]
    pub fn auth(&self) -> FirebaseAuth {
        FirebaseAuth::new(self.middleware.clone())
    }

    /// Returns a client for interacting with Firebase Cloud Messaging (FCM).
    #[cfg(feature = "messaging")]
    pub fn messaging(&self) -> FirebaseMessaging {
        FirebaseMessaging::new(self.middleware.clone())
    }

    /// Returns a client for interacting with Firebase Remote Config.
    #[cfg(feature = "remote_config")]
    pub fn remote_config(&self) -> FirebaseRemoteConfig {
        FirebaseRemoteConfig::new(self.middleware.clone())
    }

    /// Returns a client for interacting with Firebase Crashlytics.
    #[cfg(feature = "crashlytics")]
    pub fn crashlytics(&self) -> FirebaseCrashlytics {
        FirebaseCrashlytics::new(self.middleware.clone())
    }

    /// Returns a client for interacting with Cloud Firestore.
    #[cfg(feature = "firestore")]
    pub fn firestore(&self) -> FirebaseFirestore {
        FirebaseFirestore::new(self.middleware.clone())
    }

    /// Returns a client for interacting with Firebase Storage.
    #[cfg(feature = "storage")]
    pub fn storage(&self) -> FirebaseStorage {
        FirebaseStorage::new(self.middleware.clone())
    }
}
