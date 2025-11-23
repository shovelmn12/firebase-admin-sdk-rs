use super::models::{
    ArrayValue, Document, ListDocumentsResponse, MapValue, Value, ValueType,
};
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
fn convert_fields_to_serde_value(
    fields: HashMap<String, Value>,
) -> Result<SerdeValue, FirestoreError> {
    let mut map = Map::new();
    for (key, value) in fields {
        map.insert(key, convert_value_to_serde_value(value)?);
    }
    Ok(SerdeValue::Object(map))
}

fn convert_value_to_serde_value(value: Value) -> Result<SerdeValue, FirestoreError> {
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
fn convert_serializable_to_fields<T: Serialize>(
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

fn convert_serde_value_to_firestore_value(value: SerdeValue) -> Result<Value, FirestoreError> {
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
}
