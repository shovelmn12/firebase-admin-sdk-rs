pub mod auth;
pub mod core;
pub mod messaging;
pub mod remote_config;

use auth::FirebaseAuth;
use firestore::FirebaseFirestore;
use messaging::FirebaseMessaging;
use remote_config::FirebaseRemoteConfig;
use yup_oauth2::ServiceAccountKey;

pub struct FirebaseApp {
    key: ServiceAccountKey,
}

impl FirebaseApp {
    pub fn new(service_account_key: ServiceAccountKey) -> Self {
        Self {
            key: service_account_key,
        }
    }

    pub fn auth(&self) -> FirebaseAuth {
        FirebaseAuth::new(self.key.clone())
    }

    pub fn messaging(&self) -> FirebaseMessaging {
        FirebaseMessaging::new(self.key.clone())
    }

    pub fn remote_config(&self) -> FirebaseRemoteConfig {
        // The remote_config client requires a project_id. If it's missing, the SDK cannot
        // configuration error.
        FirebaseRemoteConfig::new(self.key.clone()).expect(
            "failed to create RemoteConfig client: project_id is missing from service account key",
        )
    }

    pub fn firestore(&self) -> FirebaseFirestore {
        FirebaseFirestore::new(self.key.clone())
    }
}
