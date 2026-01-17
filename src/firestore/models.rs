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

// --- Transaction Models ---

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BeginTransactionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<TransactionOptions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOptions {
    #[serde(flatten)]
    pub mode: Option<TransactionMode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TransactionMode {
    ReadOnly(ReadOnlyOptions),
    ReadWrite(ReadWriteOptions),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReadOnlyOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReadWriteOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_transaction: Option<String>, // Previous transaction ID
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BeginTransactionResponse {
    pub transaction: String, // The transaction ID (bytes as string)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RollbackRequest {
    pub transaction: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitRequest {
    pub transaction: String,
    pub writes: Vec<Write>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitResponse {
    #[serde(default)]
    pub write_results: Vec<WriteResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
    #[serde(default)]
    pub transform_results: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Write {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_mask: Option<DocumentMask>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_transforms: Option<Vec<FieldTransform>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_document: Option<Precondition>,
    #[serde(flatten)]
    pub operation: WriteOperation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum WriteOperation {
    Update(Document),
    Delete(String), // Document name
    Transform(DocumentTransform),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMask {
    pub field_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Precondition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentTransform {
    pub document: String,
    pub field_transforms: Vec<FieldTransform>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldTransform {
    pub field_path: String,
    #[serde(flatten)]
    pub transform_type: TransformType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TransformType {
    SetToServerValue(String), // e.g. "REQUEST_TIME"
    Increment(Value),
    Maximum(Value),
    Minimum(Value),
    AppendMissingElements(ArrayValue),
    RemoveAllFromArray(ArrayValue),
}

// --- Listen API Models ---

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListenRequest {
    pub database: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_target: Option<Target>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_target: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    #[serde(flatten)]
    pub target_type: Option<TargetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_token: Option<String>, // byte string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>, // timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub once: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TargetType {
    Query(QueryTarget),
    Documents(DocumentsTarget),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryTarget {
    pub parent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_query: Option<StructuredQuery>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentsTarget {
    pub documents: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListenResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_change: Option<TargetChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_change: Option<DocumentChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_delete: Option<DocumentDelete>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_remove: Option<DocumentRemove>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<ExistenceFilter>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetChange {
    #[serde(default)]
    pub target_change_type: TargetChangeType,
    #[serde(default)]
    pub target_ids: Vec<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_token: Option<String>, // byte string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TargetChangeType {
    NoChange,
    Add,
    Remove,
    Current,
    Reset,
}

impl Default for TargetChangeType {
    fn default() -> Self {
        TargetChangeType::NoChange
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<Document>,
    #[serde(default)]
    pub target_ids: Vec<i32>,
    #[serde(default)]
    pub removed_target_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDelete {
    pub document: String,
    #[serde(default)]
    pub removed_target_ids: Vec<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRemove {
    pub document: String,
    #[serde(default)]
    pub removed_target_ids: Vec<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExistenceFilter {
    pub count: i32,
    pub target_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unchanged_names: Option<BloomFilter>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BloomFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bits: Option<BitSequence>,
    pub hash_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BitSequence {
    pub bitmap: String,
    pub padding: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub details: Vec<HashMap<String, serde_json::Value>>,
}

// --- Structured Query Models ---

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StructuredQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<Projection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Vec<CollectionSelector>>,
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub where_clause: Option<QueryFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<Vec<Order>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<Cursor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<Cursor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Projection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldReference>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSelector {
    pub collection_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_descendants: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryFilter {
    #[serde(flatten)]
    pub filter_type: Option<FilterType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FilterType {
    CompositeFilter(CompositeFilter),
    FieldFilter(FieldFilter),
    UnaryFilter(UnaryFilter),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompositeFilter {
    pub op: CompositeOperator,
    pub filters: Vec<QueryFilter>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompositeOperator {
    OperatorUnspecified,
    And,
    Or,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldFilter {
    pub field: FieldReference,
    pub op: FieldOperator,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FieldOperator {
    OperatorUnspecified,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
    ArrayContains,
    In,
    ArrayContainsAny,
    NotIn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnaryFilter {
    pub op: UnaryOperator,
    pub field: FieldReference,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnaryOperator {
    OperatorUnspecified,
    IsNan,
    IsNull,
    IsNotNan,
    IsNotNull,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub field: FieldReference,
    pub direction: Direction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    DirectionUnspecified,
    Ascending,
    Descending,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldReference {
    pub field_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Cursor {
    pub values: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<bool>,
}
