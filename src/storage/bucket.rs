use crate::core::middleware::AuthMiddleware;
use crate::storage::file::File;
use reqwest_middleware::ClientWithMiddleware;

/// A reference to a Google Cloud Storage bucket.
pub struct Bucket {
    client: ClientWithMiddleware,
    base_url: String,
    name: String,
    middleware: AuthMiddleware,
}

impl Bucket {
    pub(crate) fn new(
        client: ClientWithMiddleware,
        base_url: String,
        name: String,
        middleware: AuthMiddleware,
    ) -> Self {
        Self {
            client,
            base_url,
            name,
            middleware,
        }
    }

    /// Returns the name of the bucket.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets a `File` instance that refers to the file at the specified path.
    ///
    /// # Arguments
    ///
    /// * `name` - The path to the file within the bucket (e.g., "images/profile.png").
    pub fn file(&self, name: &str) -> File {
        File::new(
            self.client.clone(),
            self.base_url.clone(),
            self.name.clone(),
            name.to_string(),
            self.middleware.clone(),
        )
    }
}
