use reqwest::{Request, Response, header};
use reqwest_middleware::{Middleware, Next};
use tokio::sync::OnceCell;
use yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};
use yup_oauth2::authenticator::Authenticator;
use hyper_rustls::HttpsConnector;
use hyper::client::HttpConnector;
use http::Extensions;

/// The concrete type of the Authenticator used by `yup-oauth2`.
///
/// In `yup-oauth2` v8 (which relies on `hyper` 0.14), the `Authenticator` is generic over the connector.
/// We use `hyper_rustls` to provide the HTTPS connector.
type AuthType = Authenticator<HttpsConnector<HttpConnector>>;

/// A middleware that handles OAuth2 authentication for Firebase requests.
///
/// This middleware intercepts outgoing requests, obtains a valid OAuth2 Bearer token
/// using the service account credentials, and injects it into the `Authorization` header.
///
/// # Lazy Initialization
///
/// The `Authenticator` is initialized lazily using `tokio::sync::OnceCell` upon the first request.
/// This allows the `FirebaseApp` constructor to remain synchronous.
pub struct AuthMiddleware {
    /// The service account key used to create the authenticator.
    key: ServiceAccountKey,
    /// A lazy-initialized authenticator instance.
    authenticator: OnceCell<AuthType>,
}

impl AuthMiddleware {
    /// Creates a new `AuthMiddleware` instance.
    ///
    /// # Arguments
    ///
    /// * `key` - The service account credentials.
    pub fn new(key: ServiceAccountKey) -> Self {
        Self {
            key,
            authenticator: OnceCell::new(),
        }
    }

    /// Retrieves a valid OAuth2 token, initializing the authenticator if necessary.
    async fn get_token(&self) -> Result<String, anyhow::Error> {
        let auth = self.authenticator.get_or_try_init(|| async {
            ServiceAccountAuthenticator::builder(self.key.clone())
                .build()
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }).await?;

        let scopes = &["https://www.googleapis.com/auth/cloud-platform", "https://www.googleapis.com/auth/firebase"];

        let token = auth.token(scopes).await?;

        Ok(token.token().ok_or_else(|| anyhow::anyhow!("No token found"))?.to_string())
    }
}

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    /// Intercepts the request to add the Authorization header.
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {

        let token = self.get_token().await.map_err(|e| {
            reqwest_middleware::Error::Middleware(anyhow::anyhow!("Failed to get auth token: {}", e))
        })?;

        req.headers_mut().insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        next.run(req, extensions).await
    }
}
