use super::listen::{listen_request, ListenStream};
use super::models::{
    ArrayValue, CollectionSelector, Document, DocumentsTarget, ListenRequest, ListDocumentsResponse,
    MapValue, QueryTarget, StructuredQuery, Target, TargetType, Value, ValueType,
};
use super::query::{run_query_request, Query};
use super::FirestoreError;
use reqwest::header;
use reqwest_middleware::ClientWithMiddleware;
use serde::de::{DeserializeOwned, Error};
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
pub(crate) fn extract_parent_and_collection(path: &str) -> Option<(String, String)> {
    // Find where "projects/" starts
    let start = path.find("projects/")?;
    let resource_path = &path[start..];

    // Last part is collection ID
    let slash_idx = resource_path.rfind('/')?;
    let collection_id = &resource_path[slash_idx + 1..];
    let parent = &resource_path[..slash_idx];

    Some((parent.to_string(), collection_id.to_string()))
}

#[derive(Clone)]
pub struct DocumentReference<'a> {
    pub(crate) client: &'a ClientWithMiddleware,
    pub(crate) path: String,
}

impl<'a> DocumentReference<'a> {
    pub async fn get<T: DeserializeOwned>(&self) -> Result<Option<T>, FirestoreError> {
        let response = self.client.get(&self.path).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
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
        let serde_value = convert_fields_to_serde_value(doc.fields)?;
        let obj = serde_json::from_value(serde_value)?;
        Ok(Some(obj))
    }

    pub async fn set<T: Serialize>(&self, value: &T) -> Result<(), FirestoreError> {
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

        Ok(())
    }

    pub async fn update<T: Serialize>(
        &self,
        value: &T,
        update_mask: Option<Vec<String>>,
    ) -> Result<(), FirestoreError> {
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

        Ok(())
    }

    pub async fn delete(&self) -> Result<(), FirestoreError> {
        let response = self.client.delete(&self.path).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FirestoreError::ApiError(format!(
                "Delete document failed {}: {}",
                status, text
            )));
        }

        Ok(())
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

#[derive(Clone)]
pub struct CollectionReference<'a> {
    pub(crate) client: &'a ClientWithMiddleware,
    pub(crate) path: String,
}

impl<'a> CollectionReference<'a> {
    pub fn doc(&self, document_id: &str) -> DocumentReference<'a> {
        DocumentReference {
            client: self.client,
            path: format!("{}/{}", self.path, document_id),
        }
    }

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

    pub async fn add<T: Serialize>(&self, value: &T) -> Result<Document, FirestoreError> {
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
        Ok(doc)
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

    /// Executes a structured query against the collection.
    ///
    /// # Arguments
    ///
    /// * `query` - The query definition.
    pub async fn query(&self, mut query: Query) -> Result<Vec<Document>, FirestoreError> {
        let database_path = extract_database_path(&self.path);
        let (parent, collection_id) = extract_parent_and_collection(&self.path).ok_or_else(|| {
            FirestoreError::ApiError("Failed to extract parent and collection ID".into())
        })?;

        // Ensure the query targets this collection
        query = query.from(&collection_id, false);

        run_query_request(self.client, &database_path, &parent, query.structured_query).await
    }
}
