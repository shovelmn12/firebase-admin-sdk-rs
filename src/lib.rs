pub mod auth;
pub mod core;
pub mod firestore;
pub mod messaging;
pub mod remote_config;

use auth::FirebaseAuth;
use core::middleware::AuthMiddleware;
use firestore::FirebaseFirestore;
use messaging::FirebaseMessaging;
use remote_config::FirebaseRemoteConfig;
use yup_oauth2::ServiceAccountKey;

pub struct FirebaseApp {
    middleware: AuthMiddleware,
}

impl FirebaseApp {
    pub fn new(service_account_key: ServiceAccountKey) -> Self {
        Self {
            middleware: AuthMiddleware::new(service_account_key),
        }
    }

    pub fn auth(&self) -> FirebaseAuth {
        FirebaseAuth::new(self.middleware.clone())
    }

    pub fn messaging(&self) -> FirebaseMessaging {
        FirebaseMessaging::new(self.middleware.clone())
    }

    pub fn remote_config(&self) -> FirebaseRemoteConfig {
        FirebaseRemoteConfig::new(self.middleware.clone())
    }

    pub fn firestore(&self) -> FirebaseFirestore {
        FirebaseFirestore::new(self.middleware.clone())
    }
}
