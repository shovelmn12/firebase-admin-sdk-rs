pub mod auth;
pub mod core;

use auth::FirebaseAuth;
use yup_oauth2::ServiceAccountKey;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use crate::core::middleware::AuthMiddleware;

pub struct FirebaseApp {
    client: ClientWithMiddleware,
    key: ServiceAccountKey,
}

impl FirebaseApp {
    pub fn new(service_account_key: ServiceAccountKey) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(AuthMiddleware::new(service_account_key.clone()))
            .build();

        Self {
            client,
            key: service_account_key,
        }
    }

    pub fn auth(&self) -> FirebaseAuth {
        let project_id = self.key.project_id.clone().unwrap_or_default();
        FirebaseAuth::new(self.client.clone(), project_id)
    }
}
