pub mod auth;
pub mod core;
pub mod messaging;
pub mod firestore;

use auth::FirebaseAuth;
use messaging::FirebaseMessaging;
use firestore::FirebaseFirestore;
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

    pub fn firestore(&self) -> FirebaseFirestore {
        FirebaseFirestore::new(self.key.clone())
    }
}
