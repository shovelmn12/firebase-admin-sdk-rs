use super::models::{
    CommitRequest, CommitResponse, Document, DocumentMask, Precondition, Write, WriteOperation,
    WriteResult,
};
use super::reference::convert_serializable_to_fields;
use super::FirestoreError;
use reqwest::header;
use reqwest_middleware::ClientWithMiddleware;
use serde::Serialize;
use std::sync::{Arc, Mutex};

/// Represents a Firestore Write Batch.
///
/// specific set of writes can be performed atomically.
///
/// # Examples
///
/// ```rust,no_run
/// # use firebase_admin_sdk::FirebaseApp;
/// # use serde_json::json;
/// # async fn run(app: FirebaseApp) -> Result<(), Box<dyn std::error::Error>> {
/// # let firestore = app.firestore();
/// let batch = firestore.batch();
/// let user1 = json!({"name": "User 1"});
/// let user2_updates = json!({"name": "User 2 Updated"});
///
/// batch.set("users/user1", &user1)?;
/// batch.update("users/user2", &user2_updates)?;
/// batch.delete("users/user3")?;
/// batch.commit().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct WriteBatch<'a> {
    client: &'a ClientWithMiddleware,
    base_url: String,
    writes: Arc<Mutex<Vec<Write>>>,
}

impl<'a> WriteBatch<'a> {
    pub(crate) fn new(client: &'a ClientWithMiddleware, base_url: String) -> Self {
        Self {
            client,
            base_url,
            writes: Arc::new(Mutex::new(Vec::new())),
        }
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
    /// If the document does not exist, the operation will fail.
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

    /// Creates a document at the given path.
    ///
    /// If the document already exists, the operation will fail.
    ///
    /// # Arguments
    ///
    /// * `document_path` - The path to the document to create.
    /// * `value` - The data to write.
    pub fn create<T: Serialize>(
        &self,
        document_path: &str,
        value: &T,
    ) -> Result<&Self, FirestoreError> {
        let fields = convert_serializable_to_fields(value)?;
        let resource_name = self.extract_resource_name(document_path);

        let write = Write {
            update_mask: None,
            update_transforms: None,
            current_document: Some(Precondition {
                exists: Some(false),
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

    fn extract_resource_name(&self, document_path: &str) -> String {
        let prefix = "https://firestore.googleapis.com/v1/";
        let base_path = self.base_url.strip_prefix(prefix).unwrap_or(&self.base_url);
        format!("{}/{}", base_path, document_path)
    }

    /// Commits the batch of writes.
    pub async fn commit(&self) -> Result<Vec<WriteResult>, FirestoreError> {
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
            transaction: None,
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
                "Commit batch failed {}: {}",
                status, text
            )));
        }

        let result: CommitResponse = response.json().await?;
        Ok(result.write_results)
    }
}
