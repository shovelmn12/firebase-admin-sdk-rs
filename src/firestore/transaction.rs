use super::models::{
    CommitRequest, CommitResponse, Document, DocumentMask, Precondition, Write, WriteOperation,
    WriteResult,
};
use super::reference::{convert_fields_to_serde_value, convert_serializable_to_fields};
use super::FirestoreError;
use reqwest::header;
use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use url::Url;

/// Represents a Firestore Transaction.
///
/// Transactions provide a way to ensure that a set of reads and writes are executed atomically.
#[derive(Clone)]
pub struct Transaction<'a> {
    client: &'a ClientWithMiddleware,
    base_url: String,
    pub transaction_id: String,
    writes: Arc<Mutex<Vec<Write>>>,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(
        client: &'a ClientWithMiddleware,
        base_url: String,
        transaction_id: String,
    ) -> Self {
        Self {
            client,
            base_url,
            transaction_id,
            writes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Reads the document at the given path.
    ///
    /// The read is performed as part of the transaction.
    ///
    /// # Arguments
    ///
    /// * `document_path` - The path to the document to read.
    pub async fn get<T: DeserializeOwned>(
        &self,
        document_path: &str,
    ) -> Result<Option<T>, FirestoreError> {
        // Construct the URL. Note that Firestore document paths in the API need to include the full resource name.
        // However, the `document_path` passed here is usually relative (e.g. "users/alice").
        // But the `base_url` is `https://firestore.../documents`.
        // So we append the relative path.
        let url = format!("{}/{}", self.base_url, document_path);
        let mut url_obj = Url::parse(&url).map_err(|e| FirestoreError::ApiError(e.to_string()))?;
        url_obj.query_pairs_mut().append_pair("transaction", &self.transaction_id);

        // Add the transaction ID query parameter
        let response = self
            .client
            .get(url_obj)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Get document in transaction failed {}: {}",
                status, text
            )));
        }

        let doc: Document = response.json().await?;
        let serde_value = convert_fields_to_serde_value(doc.fields)?;
        let obj = serde_json::from_value(serde_value)?;
        Ok(Some(obj))
    }

    /// Overwrites the document referred to by `document_path`.
    ///
    /// If the document does not exist, it will be created. If it does exist, it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `document_path` - The path to the document to write.
    /// * `value` - The data to write.
    pub fn set<T: Serialize>(
        &self,
        document_path: &str,
        value: &T,
    ) -> Result<&Self, FirestoreError> {
        let fields = convert_serializable_to_fields(value)?;
        let resource_name = self.extract_resource_name(document_path);

        let write = Write {
            update_mask: None,
            update_transforms: None,
            current_document: None,
            operation: WriteOperation::Update(Document {
                name: resource_name,
                fields,
                create_time: String::new(), // Ignored on write
                update_time: String::new(), // Ignored on write
            }),
        };

        self.writes.lock().unwrap().push(write);
        Ok(self)
    }

    /// Updates fields in the document referred to by `document_path`.
    ///
    /// If the document does not exist, the transaction will fail.
    ///
    /// # Arguments
    ///
    /// * `document_path` - The path to the document to update.
    /// * `value` - The data to update.
    pub fn update<T: Serialize>(
        &self,
        document_path: &str,
        value: &T,
    ) -> Result<&Self, FirestoreError> {
        let fields = convert_serializable_to_fields(value)?;
        let resource_name = self.extract_resource_name(document_path);

        // For update, we need to specify which fields we are updating to avoid overwriting everything else if we only pass a subset.
        // However, if the user passes a struct, we usually assume they want to update all fields present in the struct.
        // The `update` method in standard Firestore SDKs usually takes a map or key-value pairs and only updates those.
        // If the user passes a struct here, `convert_serializable_to_fields` will return all fields in that struct.
        // We should construct a FieldMask based on the keys in `fields`.

        let field_paths = fields.keys().cloned().collect();

        let write = Write {
            update_mask: Some(DocumentMask { field_paths }),
            update_transforms: None,
            current_document: Some(Precondition {
                exists: Some(true),
                update_time: None,
            }),
            operation: WriteOperation::Update(Document {
                name: resource_name,
                fields,
                create_time: String::new(),
                update_time: String::new(),
            }),
        };

        self.writes.lock().unwrap().push(write);
        Ok(self)
    }

    /// Deletes the document referred to by `document_path`.
    ///
    /// # Arguments
    ///
    /// * `document_path` - The path to the document to delete.
    pub fn delete(&self, document_path: &str) -> Result<&Self, FirestoreError> {
        let resource_name = self.extract_resource_name(document_path);

        let write = Write {
            update_mask: None,
            update_transforms: None,
            current_document: None,
            operation: WriteOperation::Delete(resource_name),
        };

        self.writes.lock().unwrap().push(write);
        Ok(self)
    }

    fn extract_resource_name(&self, document_path: &str) -> String {
        // base_url: https://firestore.googleapis.com/v1/projects/my-project/databases/(default)/documents
        // document_path: users/alice
        // result: projects/my-project/databases/(default)/documents/users/alice

        let prefix = "https://firestore.googleapis.com/v1/";
        let base_path = self.base_url.strip_prefix(prefix).unwrap_or(&self.base_url);
        format!("{}/{}", base_path, document_path)
    }

    /// Commits the transaction.
    ///
    /// This is called automatically by `run_transaction`.
    pub(crate) async fn commit(&self) -> Result<Vec<WriteResult>, FirestoreError> {
        let writes = {
            let mut guard = self.writes.lock().unwrap();
            let w = guard.clone();
            guard.clear();
            w
        };

        if writes.is_empty() {
            return Ok(Vec::new());
        }

        let url = format!("{}:commit", self.base_url.split("/documents").next().unwrap());

        let request = CommitRequest {
            transaction: Some(self.transaction_id.clone()),
            writes,
        };

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&request)?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Commit transaction failed {}: {}",
                status, text
            )));
        }

        let result: CommitResponse = response.json().await?;
        Ok(result.write_results)
    }
}
