use super::models::Document;
use super::reference::{convert_fields_to_serde_value, convert_value_to_serde_value, DocumentReference};
use super::FirestoreError;
use serde::de::DeserializeOwned;

/// A snapshot of a document in Firestore.
///
/// It contains data read from a document in your Firestore database.
/// The data can be extracted with `.data()`.
#[derive(Debug, Clone)]
pub struct DocumentSnapshot<'a> {
    pub(crate) id: String,
    pub(crate) reference: DocumentReference<'a>,
    pub(crate) document: Option<Document>,
    pub(crate) read_time: Option<String>,
}

impl<'a> DocumentSnapshot<'a> {
    /// The ID of the document.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The `DocumentReference` for the document.
    pub fn reference(&self) -> &DocumentReference<'a> {
        &self.reference
    }

    /// Returns `true` if the document exists.
    pub fn exists(&self) -> bool {
        self.document.is_some()
    }

    /// The time the document was created. Returns `None` if the document does not exist.
    pub fn create_time(&self) -> Option<&str> {
        self.document.as_ref().map(|d| d.create_time.as_str())
    }

    /// The time the document was last updated. Returns `None` if the document does not exist.
    pub fn update_time(&self) -> Option<&str> {
        self.document.as_ref().map(|d| d.update_time.as_str())
    }

    /// The time this snapshot was read.
    pub fn read_time(&self) -> Option<&str> {
        self.read_time.as_deref()
    }

    /// Retrieves all fields in the document as a specific type.
    ///
    /// Returns `Ok(None)` if the document does not exist.
    pub fn data<T: DeserializeOwned>(&self) -> Result<Option<T>, FirestoreError> {
        if let Some(doc) = &self.document {
            let serde_value = convert_fields_to_serde_value(doc.fields.clone())?;
            let obj = serde_json::from_value(serde_value)?;
            Ok(Some(obj))
        } else {
            Ok(None)
        }
    }

    /// Retrieves a specific field from the document.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the field (e.g., "address.city").
    pub fn get_field<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>, FirestoreError> {
        if let Some(doc) = &self.document {
            // Simple field access for now. Nested fields would require parsing the path.
            // For now, we only support top-level fields or simple map traversal if implemented manually.
            // TODO: Support dot notation for nested fields properly.

            if let Some(value) = doc.fields.get(path) {
                let serde_value = convert_value_to_serde_value(value.clone())?;
                let obj = serde_json::from_value(serde_value)?;
                Ok(Some(obj))
            } else {
                 // Try to traverse if dot is present?
                 // For now, just return None if not found at top level.
                 Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

/// A `QuerySnapshot` contains zero or more `DocumentSnapshot` objects.
#[derive(Debug, Clone)]
pub struct QuerySnapshot<'a> {
    pub(crate) documents: Vec<DocumentSnapshot<'a>>,
    pub(crate) read_time: Option<String>,
}

impl<'a> QuerySnapshot<'a> {
    /// The documents in this snapshot.
    pub fn documents(&self) -> &Vec<DocumentSnapshot<'a>> {
        &self.documents
    }

    /// Returns `true` if there are no documents in the snapshot.
    pub fn empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// The number of documents in the snapshot.
    pub fn size(&self) -> usize {
        self.documents.len()
    }

    /// The time this snapshot was read.
    pub fn read_time(&self) -> Option<&str> {
        self.read_time.as_deref()
    }

    /// Iterates over the document snapshots.
    pub fn iter(&self) -> std::slice::Iter<'_, DocumentSnapshot<'a>> {
        self.documents.iter()
    }
}

impl<'a> IntoIterator for &'a QuerySnapshot<'a> {
    type Item = &'a DocumentSnapshot<'a>;
    type IntoIter = std::slice::Iter<'a, DocumentSnapshot<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.documents.iter()
    }
}

/// The result of a write operation.
#[derive(Debug, Clone)]
pub struct WriteResult {
    /// The time the write occurred.
    pub write_time: String,
}
