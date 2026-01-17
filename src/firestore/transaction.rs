use super::models::{
    CommitRequest, CommitResponse, Document, Write, WriteOperation,
};
use super::reference::DocumentReference;
use super::FirestoreError;
use reqwest_middleware::ClientWithMiddleware;
use serde::Serialize;

/// A Firestore Transaction.
pub struct Transaction<'a> {
    client: &'a ClientWithMiddleware,
    base_url: String,
    transaction_id: String,
    writes: Vec<Write>,
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
            writes: Vec::new(),
        }
    }

    /// Reads the document referred to by `doc_ref`.
    ///
    /// # Arguments
    ///
    /// * `doc_ref` - The document reference to read.
    pub async fn get(&self, doc_ref: &DocumentReference<'_>) -> Result<Option<Document>, FirestoreError> {
        let response = self
            .client
            .get(&doc_ref.path)
            .query(&[("transaction", &self.transaction_id)])
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Transaction get document failed {}: {}",
                status, text
            )));
        }

        let doc: Document = response.json().await?;
        Ok(Some(doc))
    }

    /// Writes to the document referred to by `doc_ref`.
    ///
    /// This buffers the write operation until `commit()` is called.
    /// This is an overwrite operation (equivalent to `set` without merge).
    pub fn set<T: Serialize>(
        &mut self,
        doc_ref: &DocumentReference<'_>,
        value: &T,
    ) -> Result<(), FirestoreError> {
        let fields = super::reference::convert_serializable_to_fields(value)?;
        let resource_name = self.extract_resource_name(&doc_ref.path)?;

        let doc = Document {
            name: resource_name,
            fields,
            create_time: String::new(),
            update_time: String::new(),
        };

        self.writes.push(Write {
            operation: Some(WriteOperation::Update(doc)),
            update_mask: None,
            current_document: None,
        });

        Ok(())
    }

    /// Updates fields in the document referred to by `doc_ref`.
    pub fn update<T: Serialize>(
        &mut self,
        doc_ref: &DocumentReference<'_>,
        value: &T,
        update_mask: Option<Vec<String>>,
    ) -> Result<(), FirestoreError> {
        let fields = super::reference::convert_serializable_to_fields(value)?;
        let resource_name = self.extract_resource_name(&doc_ref.path)?;

        let doc = Document {
            name: resource_name,
            fields,
            create_time: String::new(),
            update_time: String::new(),
        };

        let mask = update_mask.map(|paths| super::models::DocumentMask { field_paths: paths });

        self.writes.push(Write {
            operation: Some(WriteOperation::Update(doc)),
            update_mask: mask,
            current_document: Some(super::models::Precondition {
                exists: Some(true),
                update_time: None,
            }),
        });

        Ok(())
    }

    /// Deletes the document referred to by `doc_ref`.
    pub fn delete(&mut self, doc_ref: &DocumentReference<'_>) -> Result<(), FirestoreError> {
        let resource_name = self.extract_resource_name(&doc_ref.path)?;

        self.writes.push(Write {
            operation: Some(WriteOperation::Delete(resource_name)),
            update_mask: None,
            current_document: None,
        });

        Ok(())
    }

    /// Commits the transaction.
    pub async fn commit(self) -> Result<CommitResponse, FirestoreError> {
        let url = format!("{}:commit", self.base_url);

        // Extract the database path from base_url.
        // base_url format: https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents
        // We need: projects/{project_id}/databases/(default)

        let database = if let Some(start) = self.base_url.find("projects/") {
             self.base_url[start..].trim_end_matches("/documents").to_string()
        } else {
             return Err(FirestoreError::ApiError(format!("Invalid base_url: {}", self.base_url)));
        };

        let request = CommitRequest {
            database,
            writes: self.writes,
            transaction: Some(self.transaction_id),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Transaction commit failed {}: {}",
                status, text
            )));
        }

        let result: CommitResponse = response.json().await?;
        Ok(result)
    }

    fn extract_resource_name(&self, full_path: &str) -> Result<String, FirestoreError> {
        // full_path: https://firestore.googleapis.com/v1/projects/.../documents/col/doc
        // We want: projects/.../documents/col/doc
        if let Some(start) = full_path.find("projects/") {
            Ok(full_path[start..].to_string())
        } else {
            Err(FirestoreError::ApiError(format!(
                "Invalid document path: {}",
                full_path
            )))
        }
    }
}
