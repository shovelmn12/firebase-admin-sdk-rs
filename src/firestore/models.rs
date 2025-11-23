use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub name: String,
    pub fields: HashMap<String, Value>,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    #[serde(flatten)]
    pub value_type: ValueType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ValueType {
    StringValue(String),
    IntegerValue(String), // Firestore sends integers as strings
    DoubleValue(f64),
    BooleanValue(bool),
    MapValue(MapValue),
    ArrayValue(ArrayValue),
    NullValue(()),
    TimestampValue(String),
    GeoPointValue(GeoPoint),
    BytesValue(String), // base64 encoded
    ReferenceValue(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapValue {
    pub fields: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArrayValue {
    pub values: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListDocumentsResponse {
    pub documents: Vec<Document>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}
