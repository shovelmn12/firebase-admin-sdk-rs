use crate::storage::StorageError;
use reqwest::header;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};

/// Represents a file within a Google Cloud Storage bucket.
pub struct File {
    client: ClientWithMiddleware,
    base_url: String,
    bucket_name: String,
    name: String,
}

/// Metadata for a Google Cloud Storage object.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectMetadata {
    pub name: Option<String>,
    pub bucket: Option<String>,
    pub generation: Option<String>,
    pub metageneration: Option<String>,
    pub content_type: Option<String>,
    pub time_created: Option<String>,
    pub updated: Option<String>,
    pub storage_class: Option<String>,
    pub size: Option<String>,
    pub md5_hash: Option<String>,
    pub media_link: Option<String>,
    pub content_encoding: Option<String>,
    pub content_disposition: Option<String>,
    pub cache_control: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
    pub crc32c: Option<String>,
    pub etag: Option<String>,
}

impl File {
    pub(crate) fn new(
        client: ClientWithMiddleware,
        base_url: String,
        bucket_name: String,
        name: String,
    ) -> Self {
        Self {
            client,
            base_url,
            bucket_name,
            name,
        }
    }

    /// Returns the name of the file.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the name of the bucket containing the file.
    pub fn bucket(&self) -> &str {
        &self.bucket_name
    }

    /// Uploads data to the file.
    ///
    /// This method uses the simple upload API.
    ///
    /// # Arguments
    ///
    /// * `body` - The data to upload.
    /// * `mime_type` - The MIME type of the data.
    pub async fn save(
        &self,
        body: impl Into<reqwest::Body>,
        mime_type: &str,
    ) -> Result<(), StorageError> {
        // Upload endpoint: https://storage.googleapis.com/upload/storage/v1/b/[BUCKET_NAME]/o
        // For testing purposes (or if base_url is not the default), we construct the upload URL from base_url.
        // If base_url is "https://storage.googleapis.com/storage/v1", we change it to "https://storage.googleapis.com/upload/storage/v1".
        // If it's something else (e.g. mock server), we just append /upload or similar?
        // Actually, GCS convention is tricky.
        // If standard GCS URL, we use the upload subdomain.
        // If using a mock (base_url doesn't contain storage.googleapis.com), we might want to assume the mock handles uploads under the same host but maybe different path?
        // For simplicity and enabling tests, let's trust that if the user overrides base_url, they might be pointing to an emulator or mock.

        let url = if self.base_url.contains("storage.googleapis.com/storage/v1") {
             format!(
                "https://storage.googleapis.com/upload/storage/v1/b/{}/o",
                self.bucket_name
            )
        } else {
            // Assume mock/emulator environment where we might append /upload prefix or similar relative to base?
            // Or better: replace "/storage/v1" with "/upload/storage/v1" if present.
            if self.base_url.contains("/storage/v1") {
                 let upload_base = self.base_url.replace("/storage/v1", "/upload/storage/v1");
                 format!("{}/b/{}/o", upload_base, self.bucket_name)
            } else {
                 // Fallback: just append /upload if it doesn't match known patterns?
                 // Or just use base_url as is, assuming the caller set it to the root of the API including 'upload' capability if needed?
                 // But `download` uses `base_url` too.
                 // Let's try to be smart for the mock server in tests.
                 // In tests: base_url is `http://127.0.0.1:PORT`.
                 // We want `http://127.0.0.1:PORT/upload/storage/v1...`
                 // But `download` uses `http://127.0.0.1:PORT/b/...` (which implies base_url was root-ish or included /storage/v1?)
                 // In `FirebaseStorage::new`, base_url is `https://storage.googleapis.com/storage/v1`.
                 // So `download` appends `/b/...` resulting in `.../storage/v1/b/...`.
                 // If I set mock url as base_url, say `http://host:port`, `download` does `http://host:port/b/...`.
                 // So for upload, I should probably target `http://host:port/upload/storage/v1/b/...`?
                 // Let's try prepending `/upload` to the path relative to the server root, but `base_url` might have a path.

                 // If base_url ends in `/storage/v1` (standard or emulated), switch to `/upload/storage/v1`.
                 if self.base_url.ends_with("/storage/v1") {
                     let upload_base = self.base_url.replace("/storage/v1", "/upload/storage/v1");
                     format!("{}/b/{}/o", upload_base, self.bucket_name)
                 } else {
                     // If strictly just a host, maybe we are mocking specific paths.
                     // Let's just fallback to standard behavior if we can't deduce.
                     // But for tests we need it to work.
                     // Let's assume for tests we want to hit `/upload/storage/v1` on the mock server if base_url is root.
                     // If base_url is `http://localhost:1234`, we want `http://localhost:1234/upload/storage/v1/b/...`?
                     format!("{}/upload/storage/v1/b/{}/o", self.base_url, self.bucket_name)
                 }
            }
        };

        let response = self
            .client
            .post(&url)
            .query(&[("uploadType", "media"), ("name", &self.name)])
            .header(header::CONTENT_TYPE, mime_type)
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(StorageError::ApiError(format!(
                "Upload failed {}: {}",
                status, text
            )));
        }

        Ok(())
    }

    /// Downloads the file's content.
    pub async fn download(&self) -> Result<bytes::Bytes, StorageError> {
        // Download endpoint: https://storage.googleapis.com/storage/v1/b/[BUCKET_NAME]/o/[OBJECT_NAME]?alt=media
        // Object name must be URL-encoded.
        let encoded_name = url::form_urlencoded::byte_serialize(self.name.as_bytes()).collect::<String>();
        let url = format!(
            "{}/b/{}/o/{}",
            self.base_url, self.bucket_name, encoded_name
        );

        let response = self
            .client
            .get(&url)
            .query(&[("alt", "media")])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(StorageError::ApiError(format!(
                "Download failed {}: {}",
                status, text
            )));
        }

        Ok(response.bytes().await?)
    }

    /// Deletes the file.
    pub async fn delete(&self) -> Result<(), StorageError> {
        let encoded_name = url::form_urlencoded::byte_serialize(self.name.as_bytes()).collect::<String>();
        let url = format!(
            "{}/b/{}/o/{}",
            self.base_url, self.bucket_name, encoded_name
        );

        let response = self.client.delete(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(StorageError::ApiError(format!(
                "Delete failed {}: {}",
                status, text
            )));
        }

        Ok(())
    }

    /// Gets the file's metadata.
    pub async fn get_metadata(&self) -> Result<ObjectMetadata, StorageError> {
        let encoded_name = url::form_urlencoded::byte_serialize(self.name.as_bytes()).collect::<String>();
        let url = format!(
            "{}/b/{}/o/{}",
            self.base_url, self.bucket_name, encoded_name
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(StorageError::ApiError(format!(
                "Get metadata failed {}: {}",
                status, text
            )));
        }

        Ok(response.json().await?)
    }
}
