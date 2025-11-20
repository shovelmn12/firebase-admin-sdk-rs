pub mod auth;
pub mod core;

use auth::FirebaseAuth;
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
}
