use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::Client;
use serde_json::Value;
use std::time::{Duration, Instant};
use thiserror::Error;

const GOOGLE_PUBLIC_KEYS_URL: &str = "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

#[derive(Error, Debug)]
pub enum KeyFetchError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Failed to parse keys")]
    ParseError,
}

#[derive(Clone)]
struct CachedKeys {
    keys: HashMap<String, String>,
    expires_at: Instant,
}

pub struct PublicKeyManager {
    client: Client,
    cache: Arc<RwLock<Option<CachedKeys>>>,
}

impl PublicKeyManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_key(&self, kid: &str) -> Result<String, KeyFetchError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = &*cache {
                if Instant::now() < cached.expires_at {
                    if let Some(key) = cached.keys.get(kid) {
                        return Ok(key.clone());
                    }
                }
            }
        }

        // Fetch new keys
        self.refresh_keys().await?;

        // Check cache again
        let cache = self.cache.read().await;
        if let Some(cached) = &*cache {
            cached.keys.get(kid).cloned().ok_or(KeyFetchError::ParseError)
        } else {
            Err(KeyFetchError::ParseError)
        }
    }

    async fn refresh_keys(&self) -> Result<(), KeyFetchError> {
        let response = self.client.get(GOOGLE_PUBLIC_KEYS_URL).send().await?;

        // Parse Cache-Control header
        let max_age = response.headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                s.split(',')
                    .find_map(|part| {
                        let part = part.trim();
                        if part.starts_with("max-age=") {
                            part.trim_start_matches("max-age=").parse::<u64>().ok()
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or(3600); // Default to 1 hour if missing

        let keys_json: HashMap<String, String> = response.json().await?;

        let mut cache = self.cache.write().await;
        *cache = Some(CachedKeys {
            keys: keys_json,
            expires_at: Instant::now() + Duration::from_secs(max_age),
        });

        Ok(())
    }
}
