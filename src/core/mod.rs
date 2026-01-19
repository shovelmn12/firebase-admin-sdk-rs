pub mod middleware;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FirebaseErrorResponse {
    pub error: FirebaseErrorDetails,
}

#[derive(Debug, Deserialize)]
pub struct FirebaseErrorDetails {
    pub code: u16,
    pub message: String,
    pub status: Option<String>,
    pub errors: Option<Vec<FirebaseSubError>>,
}

#[derive(Debug, Deserialize)]
pub struct FirebaseSubError {
    pub message: String,
    pub domain: Option<String>,
    pub reason: Option<String>,
}

impl FirebaseErrorResponse {
    pub fn display_message(&self) -> String {
        format!("{} (code: {})", self.error.message, self.error.code)
    }
}

pub async fn parse_error_response(response: reqwest::Response, default_msg: &str) -> String {
    let status = response.status();
    match response.json::<FirebaseErrorResponse>().await {
        Ok(error_resp) => error_resp.display_message(),
        Err(_) => format!("{}: {}", default_msg, status),
    }
}