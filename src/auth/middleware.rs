use reqwest::{Request, Response, header};
use reqwest_middleware::{Middleware, Next};
use tokio::sync::OnceCell;
use yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};
use yup_oauth2::authenticator::Authenticator;
use hyper_rustls::HttpsConnector;
use hyper::client::HttpConnector;

// The type returned by ServiceAccountAuthenticator::builder(...).build().await
// In yup-oauth2 v8 with hyper-rustls 0.26, it returns Authenticator<HttpsConnector<HttpConnector>>.
type AuthType = Authenticator<HttpsConnector<HttpConnector>>;

pub struct AuthMiddleware {
    key: ServiceAccountKey,
    authenticator: OnceCell<AuthType>,
}

impl AuthMiddleware {
    pub fn new(key: ServiceAccountKey) -> Self {
        Self {
            key,
            authenticator: OnceCell::new(),
        }
    }

    async fn get_token(&self) -> Result<String, anyhow::Error> {
        let auth = self.authenticator.get_or_try_init(|| async {
            // ServiceAccountAuthenticator::builder returns a builder.
            // .build() returns a Future that resolves to Authenticator<...>.
            ServiceAccountAuthenticator::builder(self.key.clone())
                .build()
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }).await?;

        // Scopes for Firebase
        let scopes = &["https://www.googleapis.com/auth/cloud-platform", "https://www.googleapis.com/auth/firebase"];

        let token = auth.token(scopes).await?;

        Ok(token.token().ok_or_else(|| anyhow::anyhow!("No token found"))?.to_string())
    }
}

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut task_local_extensions::Extensions,
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
