use crate::core::middleware::AuthMiddleware;
use crate::storage::StorageError;
use reqwest::header;
use reqwest_middleware::ClientWithMiddleware;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::{Pkcs1v15Sign, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, Duration};
use url::Url;

/// Represents a file within a Google Cloud Storage bucket.
pub struct File {
    client: ClientWithMiddleware,
    base_url: String,
    bucket_name: String,
    name: String,
    middleware: AuthMiddleware,
}

/// Options for generating a signed URL.
#[derive(Debug, Clone)]
pub struct GetSignedUrlOptions {
    /// The HTTP method to allow.
    pub method: SignedUrlMethod,
    /// The expiration time.
    pub expires: SystemTime,
    /// The content type (required if the client provides it).
    pub content_type: Option<String>,
}

/// Supported HTTP methods for signed URLs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignedUrlMethod {
    GET,
    PUT,
    POST,
    DELETE,
}

impl SignedUrlMethod {
    fn as_str(&self) -> &'static str {
        match self {
            SignedUrlMethod::GET => "GET",
            SignedUrlMethod::PUT => "PUT",
            SignedUrlMethod::POST => "POST",
            SignedUrlMethod::DELETE => "DELETE",
        }
    }
}

/// Metadata for a Google Cloud Storage object.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectMetadata {
    // ... (fields remain same)
    /// The name of the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The bucket containing the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,
    /// The content generation of this object. Used for object versioning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<String>,
    /// The version of the metadata for this object at this generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metageneration: Option<String>,
    /// Content-Type of the object data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// The creation time of the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<String>,
    /// The modification time of the object metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    /// Storage class of the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,
    /// Content-Length of the data in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// MD5 hash of the data; encoded using base64.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5_hash: Option<String>,
    /// Media download link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_link: Option<String>,
    /// Content-Encoding of the object data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,
    /// Content-Disposition of the object data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_disposition: Option<String>,
    /// Cache-Control directive for the object data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<String>,
    /// User-provided metadata, in key/value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
    /// CRC32c checksum, as described in RFC 4960, Appendix B; encoded using base64.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crc32c: Option<String>,
    /// HTTP 1.1 Entity tag for the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

impl File {
    pub(crate) fn new(
        client: ClientWithMiddleware,
        base_url: String,
        bucket_name: String,
        name: String,
        middleware: AuthMiddleware,
    ) -> Self {
        Self {
            client,
            base_url,
            bucket_name,
            name,
            middleware,
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

    /// Generates a V4 signed URL for accessing the file.
    ///
    /// # Arguments
    ///
    /// * `options` - The options for generating the signed URL.
    pub fn get_signed_url(&self, options: GetSignedUrlOptions) -> Result<String, StorageError> {
        let key = &self.middleware.key;
        let client_email = &key.client_email;
        let private_key_pem = &key.private_key;

        if client_email.is_empty() || private_key_pem.is_empty() {
            return Err(StorageError::ProjectIdMissing); // Using existing error enum, though maybe not precise
        }

                let now = SystemTime::now();

                

                let iso_date = chrono::DateTime::<chrono::Utc>::from(now).format("%Y%m%dT%H%M%SZ").to_string();

        
        let date_stamp = &iso_date[0..8]; // YYYYMMDD

        let credential_scope = format!("{}/auto/storage/goog4_request", date_stamp);

        let host = "storage.googleapis.com";
        let canonical_headers = format!("host:{}\n", host);
        let signed_headers = "host";

        // Canonical Resource
        let encoded_name = url::form_urlencoded::byte_serialize(self.name.as_bytes())
            .collect::<String>()
            .replace("+", "%20");

        let canonical_uri = format!("/{}/{}", self.bucket_name, encoded_name);

        // Calculate expiration in seconds from now
        let duration_seconds = options
            .expires
            .duration_since(now)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let mut query_params = vec![
            ("X-Goog-Algorithm", "GOOG4-RSA-SHA256".to_string()),
            (
                "X-Goog-Credential",
                format!("{}/{}", client_email, credential_scope),
            ),
            ("X-Goog-Date", iso_date.clone()),
            ("X-Goog-Expires", duration_seconds.to_string()),
            ("X-Goog-SignedHeaders", signed_headers.to_string()),
        ];

        query_params.sort_by(|a, b| a.0.cmp(b.0));
        
        let canonical_query_string = query_params.iter()
            .map(|(k, v)| {
                let encoded_k = url::form_urlencoded::byte_serialize(k.as_bytes()).collect::<String>();
                let encoded_v = url::form_urlencoded::byte_serialize(v.as_bytes()).collect::<String>();
                format!("{}={}", encoded_k, encoded_v)
            })
            .collect::<Vec<_>>()
            .join("&");

        let payload_hash = "UNSIGNED-PAYLOAD";

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n\n{}\n{}",
            options.method.as_str(),
            canonical_uri,
            canonical_query_string,
            canonical_headers,
            signed_headers,
            payload_hash
        );

        let algorithm = "GOOG4-RSA-SHA256";
        let request_hash = Sha256::digest(canonical_request.as_bytes());
        let request_hash_hex = hex::encode(request_hash);

        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm, iso_date, credential_scope, request_hash_hex
        );

        let hash_to_sign = Sha256::digest(string_to_sign.as_bytes());

        let priv_key = if private_key_pem.contains("BEGIN RSA PRIVATE KEY") {
            RsaPrivateKey::from_pkcs1_pem(private_key_pem).map_err(|e| {
                StorageError::ApiError(format!("Invalid private key (PKCS1): {}", e))
            })?
        } else {
            RsaPrivateKey::from_pkcs8_pem(private_key_pem).map_err(|e| {
                StorageError::ApiError(format!("Invalid private key (PKCS8): {}", e))
            })?
        };

        let signature = priv_key
            .sign(Pkcs1v15Sign::new::<Sha256>(), &hash_to_sign)
            .map_err(|e| StorageError::ApiError(format!("Signing failed: {}", e)))?;

        let signature_hex = hex::encode(signature);

        let final_url = format!(
            "https://{}{}?{}&X-Goog-Signature={}",
            host, canonical_uri, canonical_query_string, signature_hex
        );

        Ok(final_url)
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
                    format!(
                        "{}/upload/storage/v1/b/{}/o",
                        self.base_url, self.bucket_name
                    )
                }
            }
        };

        let mut url_obj = Url::parse(&url).map_err(|e| StorageError::ApiError(e.to_string()))?;
        url_obj
            .query_pairs_mut()
            .append_pair("uploadType", "media")
            .append_pair("name", &self.name);

        let response = self
            .client
            .post(url_obj)
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
        let encoded_name =
            url::form_urlencoded::byte_serialize(self.name.as_bytes()).collect::<String>();
        let url = format!(
            "{}/b/{}/o/{}",
            self.base_url, self.bucket_name, encoded_name
        );

        let mut url_obj = Url::parse(&url).map_err(|e| StorageError::ApiError(e.to_string()))?;
        url_obj.query_pairs_mut().append_pair("alt", "media");

        let response = self.client.get(url_obj).send().await?;

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
        let encoded_name =
            url::form_urlencoded::byte_serialize(self.name.as_bytes()).collect::<String>();
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
        let encoded_name =
            url::form_urlencoded::byte_serialize(self.name.as_bytes()).collect::<String>();
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

    /// Sets the file's metadata.
    ///
    /// This method uses the PATCH method to update the file's metadata.
    /// Only non-null fields in the provided `metadata` object will be updated.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata to set.
    pub async fn set_metadata(&self, metadata: &ObjectMetadata) -> Result<ObjectMetadata, StorageError> {
        let encoded_name =
            url::form_urlencoded::byte_serialize(self.name.as_bytes()).collect::<String>();
        let url = format!(
            "{}/b/{}/o/{}",
            self.base_url, self.bucket_name, encoded_name
        );

        let response = self
            .client
            .patch(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(metadata)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(StorageError::ApiError(format!(
                "Set metadata failed {}: {}",
                status, text
            )));
        }

        Ok(response.json().await?)
    }
}
