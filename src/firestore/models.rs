use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Firestore document.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// The resource name of the document.
    pub name: String,
    /// The document's fields.
    pub fields: HashMap<String, Value>,
    /// The time at which the document was created.
    pub create_time: String,
    /// The time at which the document was last changed.
    pub update_time: String,
}

/// A message that can hold any of the supported value types.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    /// The type of the value.
    #[serde(flatten)]
    pub value_type: ValueType,
}

/// The type of the value.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ValueType {
    /// A string value.
    StringValue(String),
    /// An integer value.
    IntegerValue(String), // Firestore sends integers as strings
    /// A double value.
    DoubleValue(f64),
    /// A boolean value.
    BooleanValue(bool),
    /// A map value.
    MapValue(MapValue),
    /// An array value.
    ArrayValue(ArrayValue),
    /// A null value.
    NullValue(()),
    /// A timestamp value.
    TimestampValue(String),
    /// A geo point value.
    GeoPointValue(GeoPoint),
    /// A bytes value (base64 encoded).
    BytesValue(String), // base64 encoded
    /// A reference to a document.
    ReferenceValue(String),
}

/// A map value.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapValue {
    /// The map's fields.
    pub fields: HashMap<String, Value>,
}

/// An array value.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArrayValue {
    /// The array's values.
    pub values: Vec<Value>,
}

/// An object representing a latitude/longitude pair.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeoPoint {
    /// The latitude in degrees.
    pub latitude: f64,
    /// The longitude in degrees.
    pub longitude: f64,
}

/// The response from listing documents.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListDocumentsResponse {
    /// The documents found.
    pub documents: Vec<Document>,
    /// The next page token.
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

// --- Transaction Models ---

/// A request to begin a transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BeginTransactionRequest {
    /// Options for the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<TransactionOptions>,
}

/// Options for a transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOptions {
    /// The mode of the transaction.
    #[serde(flatten)]
    pub mode: Option<TransactionMode>,
}

/// The mode of a transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TransactionMode {
    /// Read-only transaction.
    ReadOnly(ReadOnlyOptions),
    /// Read-write transaction.
    ReadWrite(ReadWriteOptions),
}

/// Options for a read-only transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReadOnlyOptions {
    /// Reads documents at the given time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>,
}

/// Options for a read-write transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReadWriteOptions {
    /// An optional transaction to retry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_transaction: Option<String>, // Previous transaction ID
}

/// The response from beginning a transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BeginTransactionResponse {
    /// The transaction ID (bytes as string).
    pub transaction: String,
}

/// A request to rollback a transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RollbackRequest {
    /// The transaction ID to rollback.
    pub transaction: String,
}

/// A request to commit a transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitRequest {
    /// The transaction ID to commit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<String>,
    /// The writes to apply.
    pub writes: Vec<Write>,
}

/// The response from committing a transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommitResponse {
    /// The result of the writes.
    #[serde(default)]
    pub write_results: Vec<WriteResult>,
    /// The time at which the commit occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_time: Option<String>,
}

/// The result of a single write.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult {
    /// The time at which the document was updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
    /// The results of applying each transform.
    #[serde(default)]
    pub transform_results: Vec<Value>,
}

/// A write operation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Write {
    /// The fields to update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_mask: Option<DocumentMask>,
    /// The transforms to perform after update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_transforms: Option<Vec<FieldTransform>>,
    /// An optional precondition on the document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_document: Option<Precondition>,
    /// The operation to perform.
    #[serde(flatten)]
    pub operation: WriteOperation,
}

/// The type of write operation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum WriteOperation {
    /// Updates a document.
    Update(Document),
    /// Deletes a document.
    Delete(String), // Document name
    /// Applies a transformation to a document.
    Transform(DocumentTransform),
}

/// A set of field paths on a document.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMask {
    /// The list of field paths in the mask.
    pub field_paths: Vec<String>,
}

/// A precondition on a document, used for conditional writes.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Precondition {
    /// When set to `true`, the target document must exist.
    /// When set to `false`, the target document must not exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,
    /// When set, the target document must exist and have been last updated at that time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
}

/// A transformation of a document.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentTransform {
    /// The name of the document to transform.
    pub document: String,
    /// The list of transformations to apply to the fields of the document.
    pub field_transforms: Vec<FieldTransform>,
}

/// A transformation of a field of the document.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldTransform {
    /// The path of the field.
    pub field_path: String,
    /// The transformation to apply.
    #[serde(flatten)]
    pub transform_type: TransformType,
}

/// The type of transformation to apply.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TransformType {
    /// Sets the field to the given server value.
    SetToServerValue(String), // e.g. "REQUEST_TIME"
    /// Adds the given value to the field's current value.
    Increment(Value),
    /// Sets the field to the maximum of its current value and the given value.
    Maximum(Value),
    /// Sets the field to the minimum of its current value and the given value.
    Minimum(Value),
    /// Appends the given elements in order if they are not already present in the current array value.
    AppendMissingElements(ArrayValue),
    /// Removes all of the given elements from the array in the field.
    RemoveAllFromArray(ArrayValue),
}

// --- Listen API Models ---

/// A request to listen to changes in documents.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListenRequest {
    /// The database name.
    pub database: String,
    /// A target to add to this stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_target: Option<Target>,
    /// The ID of a target to remove from this stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_target: Option<i32>,
    /// Labels associated with this request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
}

/// A specification of a set of documents to listen to.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    /// The type of target to listen to.
    #[serde(flatten)]
    pub target_type: Option<TargetType>,
    /// A resume token from a prior `ListenResponse`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_token: Option<String>, // byte string
    /// Start listening after a specific `read_time`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>, // timestamp
    /// The target ID that identifies the target on the stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<i32>,
    /// If the target should be removed once it is current and consistent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub once: Option<bool>,
    /// The number of documents that last matched the query at the resume token or read time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_count: Option<i32>,
}

/// The type of target to listen to.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TargetType {
    /// A target specified by a query.
    Query(QueryTarget),
    /// A target specified by a set of document names.
    Documents(DocumentsTarget),
}

/// A target specified by a query.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryTarget {
    /// The parent resource name.
    pub parent: String,
    /// The structured query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_query: Option<StructuredQuery>,
}

/// A target specified by a set of document names.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentsTarget {
    /// The names of the documents to retrieve.
    pub documents: Vec<String>,
}

/// The response for `ListenRequest`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListenResponse {
    /// Targets have changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_change: Option<TargetChange>,
    /// A Document has changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_change: Option<DocumentChange>,
    /// A Document has been deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_delete: Option<DocumentDelete>,
    /// A Document has been removed from a target (but not deleted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_remove: Option<DocumentRemove>,
    /// A filter to apply to the set of documents matching the targets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<ExistenceFilter>,
}

/// Targets being watched have changed.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetChange {
    /// The type of change that occurred.
    #[serde(default)]
    pub target_change_type: TargetChangeType,
    /// The target IDs of targets that have changed.
    #[serde(default)]
    pub target_ids: Vec<i32>,
    /// The error that resulted in this change, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Status>,
    /// A token that can be used to resume the stream for the given `target_ids`,
    /// or all targets if `target_ids` is empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_token: Option<String>, // byte string
    /// The consistent `read_time` for the given `target_ids`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>,
}

/// The type of change.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TargetChangeType {
    /// No change has occurred.
    NoChange,
    /// The targets have been added.
    Add,
    /// The targets have been removed.
    Remove,
    /// The targets reflect all changes committed before the targets were added
    /// to the stream.
    Current,
    /// The targets have been reset.
    Reset,
}

impl Default for TargetChangeType {
    fn default() -> Self {
        TargetChangeType::NoChange
    }
}

/// A Document has changed.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChange {
    /// The new state of the document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<Document>,
    /// A set of target IDs of targets that match this document.
    #[serde(default)]
    pub target_ids: Vec<i32>,
    /// A set of target IDs for targets that no longer match this document.
    #[serde(default)]
    pub removed_target_ids: Vec<i32>,
}

/// A Document has been deleted.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDelete {
    /// The resource name of the Document that was deleted.
    pub document: String,
    /// A set of target IDs for targets that previously matched this entity.
    #[serde(default)]
    pub removed_target_ids: Vec<i32>,
    /// The read timestamp at which the delete was observed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>,
}

/// A Document has been removed from the view of the targets.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRemove {
    /// The resource name of the Document that has gone out of view.
    pub document: String,
    /// A set of target IDs for targets that previously matched this document.
    #[serde(default)]
    pub removed_target_ids: Vec<i32>,
    /// The read timestamp at which the remove was observed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>,
}

/// A Digest of documents that match the given target.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExistenceFilter {
    /// The total count of documents that match target_id.
    pub count: i32,
    /// The target ID to which this filter applies.
    pub target_id: i32,
    /// A Bloom filter for the documents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unchanged_names: Option<BloomFilter>,
}

/// A Bloom filter.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BloomFilter {
    /// The bloom filter data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bits: Option<BitSequence>,
    /// The number of hashes used by the algorithm.
    pub hash_count: i32,
}

/// A sequence of bits.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BitSequence {
    /// The bytes that encode the bit sequence.
    pub bitmap: String,
    /// The number of padding bits in the last byte.
    pub padding: i32,
}

/// The RPC status.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The status code.
    pub code: i32,
    /// The error message.
    pub message: String,
    /// A list of messages that carry the error details.
    #[serde(default)]
    pub details: Vec<HashMap<String, serde_json::Value>>,
}

// --- Structured Query Models ---

/// A Firestore query.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StructuredQuery {
    /// The projection to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<Projection>,
    /// The collections to query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Vec<CollectionSelector>>,
    /// The filter to apply.
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub where_clause: Option<QueryFilter>,
    /// The order to apply to the query results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<Vec<Order>>,
    /// A potential prefix of a position in the result set to start the query at.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<Cursor>,
    /// A potential prefix of a position in the result set to end the query at.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<Cursor>,
    /// The number of results to skip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    /// The maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

/// The request for `runQuery`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunQueryRequest {
    /// The parent resource name.
    pub parent: String,
    /// The structured query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_query: Option<StructuredQuery>,
    // TODO: Add support for transactions
    // pub transaction: Option<String>,
    // pub new_transaction: Option<TransactionOptions>,
    // pub read_time: Option<String>,
}

/// The response for `runQuery`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunQueryResponse {
    /// The transaction that was started or is being used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<String>,
    /// A query result, not set when reporting partial progress.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<Document>,
    /// The time at which the document was read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<String>,
    /// The number of results that have been skipped due to an offset between
    /// the last response and the current response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_results: Option<i32>,
}

/// The projection of document's fields to return.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Projection {
    /// The fields to return. If empty, all fields are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldReference>>,
}

/// A selection of a collection, such as `messages as m1`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSelector {
    /// The collection ID.
    pub collection_id: String,
    /// When false, selects only collections that are immediate children of
    /// the `parent` specified in the containing `RunQueryRequest`.
    /// When true, selects all descendant collections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_descendants: Option<bool>,
}

/// A filter.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryFilter {
    /// The type of filter.
    #[serde(flatten)]
    pub filter_type: Option<FilterType>,
}

/// The type of filter.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FilterType {
    /// A composite filter.
    CompositeFilter(CompositeFilter),
    /// A filter on a document field.
    FieldFilter(FieldFilter),
    /// A filter that takes exactly one argument.
    UnaryFilter(UnaryFilter),
}

/// A filter that merges multiple other filters using the given operator.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompositeFilter {
    /// The operator for combining multiple filters.
    pub op: CompositeOperator,
    /// The list of filters to combine.
    pub filters: Vec<QueryFilter>,
}

/// A composite operator.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompositeOperator {
    /// Unspecified operator.
    OperatorUnspecified,
    /// The results are required to satisfy each of the combined filters.
    And,
    /// The results are required to satisfy at least one of the combined filters.
    Or,
}

/// A filter on a specific field.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldFilter {
    /// The field to filter by.
    pub field: FieldReference,
    /// The operator to use for comparison.
    pub op: FieldOperator,
    /// The value to compare to.
    pub value: Value,
}

/// A field filter operator.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FieldOperator {
    /// Unspecified operator.
    OperatorUnspecified,
    /// Less than.
    LessThan,
    /// Less than or equal.
    LessThanOrEqual,
    /// Greater than.
    GreaterThan,
    /// Greater than or equal.
    GreaterThanOrEqual,
    /// Equal.
    Equal,
    /// Not equal.
    NotEqual,
    /// Array contains.
    ArrayContains,
    /// In.
    In,
    /// Array contains any.
    ArrayContainsAny,
    /// Not in.
    NotIn,
}

/// A filter with a single operand.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnaryFilter {
    /// The unary operator.
    pub op: UnaryOperator,
    /// The field to which to apply the operator.
    pub field: FieldReference,
}

/// A unary operator.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnaryOperator {
    /// Unspecified operator.
    OperatorUnspecified,
    /// IS NAN.
    IsNan,
    /// IS NULL.
    IsNull,
    /// IS NOT NAN.
    IsNotNan,
    /// IS NOT NULL.
    IsNotNull,
}

/// An order on a field.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// The field to order by.
    pub field: FieldReference,
    /// The direction to order by.
    pub direction: Direction,
}

/// A sort direction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    /// Unspecified direction.
    DirectionUnspecified,
    /// Ascending.
    Ascending,
    /// Descending.
    Descending,
}

/// A reference to a field.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldReference {
    /// The path of the field.
    pub field_path: String,
}

/// A position in a result set.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Cursor {
    /// The values that represent the position, in the order they appear in
    /// the order by clause of a query.
    pub values: Vec<Value>,
    /// If the position is just before or just after the given values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<bool>,
}

// --- List Collections Models ---

/// The request for `listCollectionIds`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListCollectionIdsRequest {
    /// The maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
    /// A page token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

/// The response for `listCollectionIds`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListCollectionIdsResponse {
    /// The collection IDs.
    #[serde(default)]
    pub collection_ids: Vec<String>,
    /// The next page token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}
