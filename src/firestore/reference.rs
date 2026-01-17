use super::listen::{listen_request, ListenStream};
use super::models::{
    ArrayValue, CollectionSelector, Document, DocumentsTarget, FieldOperator, ListenRequest,
    ListDocumentsResponse, MapValue, QueryTarget, StructuredQuery, Target, TargetType, Value,
    ValueType,
};
use super::query::Query;
use super::snapshot::{DocumentSnapshot, WriteResult};
use super::FirestoreError;
use reqwest::header;
use reqwest_middleware::ClientWithMiddleware;
use serde::de::Error;
use serde::ser::Error as SerError;
use serde::Serialize;
use serde_json::map::Map;
use serde_json::Value as SerdeValue;
use std::collections::HashMap;

// Helper to convert Firestore's value map to a standard serde_json::Value
pub(crate) fn convert_fields_to_serde_value(
    fields: HashMap<String, Value>,
) -> Result<SerdeValue, FirestoreError> {
    let mut map = Map::new();
    for (key, value) in fields {
        map.insert(key, convert_value_to_serde_value(value)?);
    }
    Ok(SerdeValue::Object(map))
}

pub(crate) fn convert_value_to_serde_value(value: Value) -> Result<SerdeValue, FirestoreError> {
    use serde_json::json;
    Ok(match value.value_type {
        ValueType::StringValue(s) => SerdeValue::String(s),
        ValueType::IntegerValue(s) => {
            let i: i64 = s.parse().map_err(|e| {
                <serde_json::Error as Error>::custom(format!(
                    "Failed to parse integer string '{}': {}",
                    s, e
                ))
            })?;
            SerdeValue::Number(i.into())
        }
        ValueType::DoubleValue(d) => SerdeValue::Number(
            serde_json::Number::from_f64(d).ok_or_else(|| {
                <serde_json::Error as Error>::custom(format!("Invalid f64 value: {}", d))
            })?,
        ),
        ValueType::BooleanValue(b) => SerdeValue::Bool(b),
        ValueType::MapValue(map_value) => convert_fields_to_serde_value(map_value.fields)?,
        ValueType::ArrayValue(array_value) => {
            let values = array_value
                .values
                .into_iter()
                .map(convert_value_to_serde_value)
                .collect::<Result<Vec<_>, _>>()?;
            SerdeValue::Array(values)
        }
        ValueType::NullValue(_) => SerdeValue::Null,
        ValueType::TimestampValue(s) => SerdeValue::String(s),
        ValueType::GeoPointValue(gp) => {
            json!({ "latitude": gp.latitude, "longitude": gp.longitude })
        }
        ValueType::BytesValue(s) => SerdeValue::String(s),
        ValueType::ReferenceValue(s) => SerdeValue::String(s),
    })
}

// Helper to convert a serializable Rust struct to Firestore's value map
pub(crate) fn convert_serializable_to_fields<T: Serialize>(
    value: &T,
) -> Result<HashMap<String, Value>, FirestoreError> {
    let serde_value = serde_json::to_value(value)?;
    if let SerdeValue::Object(map) = serde_value {
        let mut fields = HashMap::new();
        for (k, v) in map {
            fields.insert(k, convert_serde_value_to_firestore_value(v)?);
        }
        Ok(fields)
    } else {
        Err(FirestoreError::SerializationError(SerError::custom(
            "Can only set objects as documents",
        )))
    }
}

pub(crate) fn convert_serde_value_to_firestore_value(
    value: SerdeValue,
) -> Result<Value, FirestoreError> {
    let value_type = match value {
        SerdeValue::Null => ValueType::NullValue(()),
        SerdeValue::Bool(b) => ValueType::BooleanValue(b),
        SerdeValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                ValueType::IntegerValue(i.to_string())
            } else if let Some(f) = n.as_f64() {
                ValueType::DoubleValue(f)
            } else {
                return Err(FirestoreError::SerializationError(SerError::custom(
                    format!("Unsupported number type: {}", n)
                )));
            }
        }
        SerdeValue::String(s) => ValueType::StringValue(s),
        SerdeValue::Array(a) => {
            let values = a
                .into_iter()
                .map(convert_serde_value_to_firestore_value)
                .collect::<Result<Vec<_>, _>>()?;
            ValueType::ArrayValue(ArrayValue { values })
        }
        SerdeValue::Object(o) => {
            let mut fields = HashMap::new();
            for (k, v) in o {
                fields.insert(k, convert_serde_value_to_firestore_value(v)?);
            }
            ValueType::MapValue(MapValue { fields })
        }
    };
    Ok(Value { value_type })
}

// Helper to extract project and database from a path
// Path format: projects/{project_id}/databases/(default)/documents/...
pub(crate) fn extract_database_path(path: &str) -> String {
    let parts: Vec<&str> = path.split("/documents").collect();
    if parts.len() > 0 {
        parts[0].to_string()
    } else {
        // Fallback
        path.to_string()
    }
}

// Helper to extract parent path and collection ID
// Input: .../documents/users
// Output: (parent_path, "users") where parent_path is relative (projects/...)
fn extract_parent_and_collection(path: &str) -> Option<(String, String)> {
    // Find where "projects/" starts
    let start = path.find("projects/")?;
    let resource_path = &path[start..];

    // Last part is collection ID
    let slash_idx = resource_path.rfind('/')?;
    let collection_id = &resource_path[slash_idx + 1..];
    let parent = &resource_path[..slash_idx];

    Some((parent.to_string(), collection_id.to_string()))
}

/// A reference to a document in a Firestore database.
#[derive(Clone, Debug)]
pub struct DocumentReference<'a> {
    pub(crate) client: &'a ClientWithMiddleware,
    pub(crate) path: String,
}

impl<'a> DocumentReference<'a> {
    /// Reads the document referenced by this `DocumentReference`.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `DocumentSnapshot`.
    pub async fn get(&self) -> Result<DocumentSnapshot<'a>, FirestoreError> {
        let response = self.client.get(&self.path).send().await?;

        // Extract ID from path
        let id = self.path.split('/').last().unwrap_or_default().to_string();

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(DocumentSnapshot {
                id,
                reference: self.clone(),
                document: None,
                read_time: None, // We don't get read time on 404 easily unless we parse error body
            });
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Get document failed {}: {}",
                status, text
            )));
        }

        let doc: Document = response.json().await?;
        let read_time = Some(chrono::Utc::now().to_rfc3339()); // Approx read time as header parsing is manual

        Ok(DocumentSnapshot {
            id,
            reference: self.clone(),
            document: Some(doc),
            read_time,
        })
    }

    /// Gets a `CollectionReference` instance that refers to the subcollection at the specified path.
    pub fn collection(&self, collection_id: &str) -> CollectionReference<'a> {
        CollectionReference {
            client: self.client,
            path: format!("{}/{}", self.path, collection_id),
        }
    }

    /// Writes to the document referred to by this `DocumentReference`.
    ///
    /// If the document does not exist, it will be created. If it does exist, it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `value` - The data to write to the document.
    pub async fn set<T: Serialize>(&self, value: &T) -> Result<WriteResult, FirestoreError> {
        let url = self.path.clone();

        let fields = convert_serializable_to_fields(value)?;

        let body = serde_json::to_vec(&serde_json::json!({ "fields": fields }))?;

        let response = self
            .client
            .patch(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Set document failed {}: {}",
                status, text
            )));
        }

        let doc: Document = response.json().await?;
        Ok(WriteResult {
            write_time: doc.update_time,
        })
    }

    /// Updates fields in the document referred to by this `DocumentReference`.
    ///
    /// If the document does not exist, the update will fail.
    ///
    /// # Arguments
    ///
    /// * `value` - The data to update.
    /// * `update_mask` - An optional list of field paths to update. If provided, only the fields in the mask will be updated.
    pub async fn update<T: Serialize>(
        &self,
        value: &T,
        update_mask: Option<Vec<String>>,
    ) -> Result<WriteResult, FirestoreError> {
        let fields = convert_serializable_to_fields(value)?;

        let mut url = self.path.clone();
        if let Some(mask) = update_mask {
            url.push('?');
            for (i, field) in mask.iter().enumerate() {
                if i > 0 {
                    url.push('&');
                }
                url.push_str(&format!("updateMask.fieldPaths={}", field));
            }
        }

        let body = serde_json::to_vec(&serde_json::json!({ "fields": fields }))?;

        let response = self
            .client
            .patch(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Update document failed {}: {}",
                status, text
            )));
        }

        let doc: Document = response.json().await?;
        Ok(WriteResult {
            write_time: doc.update_time,
        })
    }

    /// Deletes the document referred to by this `DocumentReference`.
    pub async fn delete(&self) -> Result<WriteResult, FirestoreError> {
        let response = self.client.delete(&self.path).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Delete document failed {}: {}",
                status, text
            )));
        }

        // Delete returns an empty object on success, or a status.
        // We can synthesize a write time or check if headers provide one?
        // Firestore REST API delete returns Empty.
        // So we might default to current time.
        Ok(WriteResult {
            write_time: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Listens to changes to the document.
    pub async fn listen(&self) -> Result<ListenStream, FirestoreError> {
        let database = extract_database_path(&self.path);

        let target = Target {
            target_type: Some(TargetType::Documents(DocumentsTarget {
                documents: vec![self.path.clone()],
            })),
            target_id: Some(1), // Arbitrary ID
            resume_token: None,
            read_time: None,
            once: None,
            expected_count: None,
        };

        let request = ListenRequest {
            database: database.clone(),
            add_target: Some(target),
            remove_target: None,
            labels: None,
        };

        listen_request(self.client, &database, &request).await
    }
}

/// A reference to a collection in a Firestore database.
#[derive(Clone, Debug)]
pub struct CollectionReference<'a> {
    pub(crate) client: &'a ClientWithMiddleware,
    pub(crate) path: String,
}

impl<'a> CollectionReference<'a> {
    /// Gets a `DocumentReference` for the document within the collection with the specified ID.
    ///
    /// # Arguments
    ///
    /// * `document_id` - The ID of the document.
    pub fn doc(&self, document_id: &str) -> DocumentReference<'a> {
        DocumentReference {
            client: self.client,
            path: format!("{}/{}", self.path, document_id),
        }
    }

    /// Lists documents in this collection.
    pub async fn list_documents(&self) -> Result<ListDocumentsResponse, FirestoreError> {
        let response = self.client.get(&self.path).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "List documents failed {}: {}",
                status, text
            )));
        }

        let list: ListDocumentsResponse = response.json().await?;
        Ok(list)
    }

    /// Adds a new document to this collection with an auto-generated ID.
    ///
    /// # Arguments
    ///
    /// * `value` - The data to write to the new document.
    pub async fn add<T: Serialize>(&self, value: &T) -> Result<DocumentReference<'a>, FirestoreError> {
        let fields = convert_serializable_to_fields(value)?;
        let body = serde_json::to_vec(&serde_json::json!({ "fields": fields }))?;

        let response = self
            .client
            .post(&self.path)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Add document failed {}: {}",
                status, text
            )));
        }

        let doc: Document = response.json().await?;
        Ok(DocumentReference {
            client: self.client,
            path: doc.name,
        })
    }

    /// Creates and returns a new `Query` with the additional filter.
    pub fn where_filter<T: Serialize>(
        &self,
        field: &str,
        op: FieldOperator,
        value: T,
    ) -> Result<Query<'a>, FirestoreError> {
        self.query().where_filter(field, op, value)
    }

    /// Creates and returns a new `Query` that's additionally sorted by the specified field.
    pub fn order_by(&self, field: &str, direction: super::models::Direction) -> Query<'a> {
        self.query().order_by(field, direction)
    }

    /// Creates and returns a new `Query` that only returns the first matching documents.
    pub fn limit(&self, limit: i32) -> Query<'a> {
        self.query().limit(limit)
    }

    /// Creates and returns a new `Query` that skips the first matching documents.
    pub fn offset(&self, offset: i32) -> Query<'a> {
        self.query().offset(offset)
    }

    fn query(&self) -> Query<'a> {
        let (parent, collection_id) = extract_parent_and_collection(&self.path)
            .expect("Collection path should be valid");

        Query::new(self.client, parent, collection_id)
    }

    /// Listens to changes in the collection.
    pub async fn listen(&self) -> Result<ListenStream, FirestoreError> {
        let database = extract_database_path(&self.path);
        let (parent, collection_id) = extract_parent_and_collection(&self.path).ok_or_else(|| {
            FirestoreError::ApiError("Failed to extract parent and collection ID".into())
        })?;

        let query_target = QueryTarget {
            parent,
            structured_query: Some(StructuredQuery {
                from: Some(vec![CollectionSelector {
                    collection_id,
                    all_descendants: None,
                }]),
                select: None,
                where_clause: None,
                order_by: None,
                start_at: None,
                end_at: None,
                offset: None,
                limit: None,
            }),
        };

        let target = Target {
            target_type: Some(TargetType::Query(query_target)),
            target_id: Some(1), // Arbitrary ID
            resume_token: None,
            read_time: None,
            once: None,
            expected_count: None,
        };

        let request = ListenRequest {
            database: database.clone(),
            add_target: Some(target),
            remove_target: None,
            labels: None,
        };

        listen_request(self.client, &database, &request).await
    }
}
