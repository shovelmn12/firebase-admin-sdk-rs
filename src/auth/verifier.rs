use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use crate::auth::keys::{PublicKeyManager, KeyFetchError};
use thiserror::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Error, Debug)]
pub enum TokenVerificationError {
    #[error("Key fetch error: {0}")]
    KeyFetchError(#[from] KeyFetchError),
    #[error("JWT validation error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Token expired")]
    Expired,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirebaseTokenClaims {
    pub aud: String,
    pub iss: String,
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub auth_time: usize,
    pub user_id: String,
    #[serde(flatten)]
    pub claims: serde_json::Map<String, serde_json::Value>,
}

pub struct IdTokenVerifier {
    project_id: String,
    key_manager: PublicKeyManager,
}

impl IdTokenVerifier {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            key_manager: PublicKeyManager::new(),
        }
    }

    pub async fn verify_token(&self, token: &str) -> Result<FirebaseTokenClaims, TokenVerificationError> {
        // 1. Decode header to get kid
        let header = decode_header(token)?;
        let kid = header.kid.ok_or_else(|| TokenVerificationError::InvalidToken("Missing kid in header".to_string()))?;

        // 2. Get public key
        let public_key_pem = self.key_manager.get_key(&kid).await?;
        let key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())?;

        // 3. Configure validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.project_id]);
        validation.set_issuer(&[format!("https://securetoken.google.com/{}", self.project_id)]);

        // 4. Verify
        let token_data = decode::<FirebaseTokenClaims>(token, &key, &validation)?;
        let claims = token_data.claims;

        // 5. Additional validations (sub not empty, auth_time < now)
        if claims.sub.is_empty() {
            return Err(TokenVerificationError::InvalidToken("Subject (sub) claim must not be empty".to_string()));
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        // Allowing some clock skew? jsonwebtoken handles exp/iat with leeway.
        // auth_time validation usually not strictly enforced by jsonwebtoken default.
        if claims.auth_time > now + 300 { // 5 minutes future skew tolerance
             return Err(TokenVerificationError::InvalidToken("Auth time is in the future".to_string()));
        }

        Ok(claims)
    }
}
