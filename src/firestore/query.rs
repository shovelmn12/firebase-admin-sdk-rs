use super::models::{
    CollectionSelector, CompositeFilter, CompositeOperator, Direction, FieldFilter, FieldOperator,
    FieldReference, FilterType, Order, Projection, QueryFilter, StructuredQuery, RunQueryRequest,
    RunQueryResponse, Document,
};
use super::reference::convert_serde_value_to_firestore_value;
use super::FirestoreError;
use reqwest_middleware::ClientWithMiddleware;
use serde::Serialize;

/// A builder for creating Firestore queries.
#[derive(Clone, Debug, Default)]
pub struct Query {
    pub(crate) structured_query: StructuredQuery,
}

impl Query {
    /// Creates a new empty Query.
    pub fn new() -> Self {
        Self {
            structured_query: StructuredQuery {
                select: None,
                from: None,
                where_clause: None,
                order_by: None,
                start_at: None,
                end_at: None,
                offset: None,
                limit: None,
            },
        }
    }

    /// sets the collection to query.
    ///
    /// This corresponds to the `from` clause in StructuredQuery.
    /// Typically, this is set automatically when using `CollectionReference`.
    pub fn from(mut self, collection_id: &str, all_descendants: bool) -> Self {
        self.structured_query.from = Some(vec![CollectionSelector {
            collection_id: collection_id.to_string(),
            all_descendants: if all_descendants { Some(true) } else { None },
        }]);
        self
    }

    /// Adds a filter to the query.
    ///
    /// # Arguments
    ///
    /// * `field` - The field path to filter on.
    /// * `op` - The operator (e.g., "==", ">", "array-contains").
    /// * `value` - The value to compare against.
    pub fn filter<T: Serialize>(mut self, field: &str, op: &str, value: T) -> Result<Self, FirestoreError> {
        let serde_value = serde_json::to_value(value)?;
        let firestore_value = convert_serde_value_to_firestore_value(serde_value)?;

        let field_filter = match op {
            "==" => self.create_field_filter(field, FieldOperator::Equal, firestore_value),
            "!=" => self.create_field_filter(field, FieldOperator::NotEqual, firestore_value),
            "<" => self.create_field_filter(field, FieldOperator::LessThan, firestore_value),
            "<=" => self.create_field_filter(field, FieldOperator::LessThanOrEqual, firestore_value),
            ">" => self.create_field_filter(field, FieldOperator::GreaterThan, firestore_value),
            ">=" => self.create_field_filter(field, FieldOperator::GreaterThanOrEqual, firestore_value),
            "array-contains" => self.create_field_filter(field, FieldOperator::ArrayContains, firestore_value),
            "in" => self.create_field_filter(field, FieldOperator::In, firestore_value),
            "array-contains-any" => self.create_field_filter(field, FieldOperator::ArrayContainsAny, firestore_value),
            "not-in" => self.create_field_filter(field, FieldOperator::NotIn, firestore_value),
            _ => return Err(FirestoreError::ApiError(format!("Unsupported operator: {}", op))),
        };

        // If there is already a filter, combine with AND
        if let Some(existing_filter) = self.structured_query.where_clause {
            // Check if existing is Composite AND
            let new_filter_wrapper = QueryFilter {
                filter_type: Some(FilterType::FieldFilter(field_filter)),
            };

            let combined = match existing_filter.filter_type {
                Some(FilterType::CompositeFilter(mut cf)) if matches!(cf.op, CompositeOperator::And) => {
                    cf.filters.push(new_filter_wrapper);
                    cf
                }
                Some(_) => {
                    CompositeFilter {
                        op: CompositeOperator::And,
                        filters: vec![existing_filter, new_filter_wrapper],
                    }
                }
                None => {
                     CompositeFilter {
                        op: CompositeOperator::And,
                        filters: vec![new_filter_wrapper],
                    }
                }
            };

            self.structured_query.where_clause = Some(QueryFilter {
                filter_type: Some(FilterType::CompositeFilter(combined)),
            });

        } else {
            self.structured_query.where_clause = Some(QueryFilter {
                filter_type: Some(FilterType::FieldFilter(field_filter)),
            });
        }

        Ok(self)
    }

    fn create_field_filter(&self, field: &str, op: FieldOperator, value: super::models::Value) -> FieldFilter {
        FieldFilter {
            field: FieldReference {
                field_path: field.to_string(),
            },
            op,
            value,
        }
    }

    /// Sorts the results.
    pub fn order_by(mut self, field: &str, direction: &str) -> Result<Self, FirestoreError> {
        let dir = match direction.to_lowercase().as_str() {
            "asc" | "ascending" => Direction::Ascending,
            "desc" | "descending" => Direction::Descending,
             _ => return Err(FirestoreError::ApiError(format!("Invalid direction: {}", direction))),
        };

        let order = Order {
            field: FieldReference {
                field_path: field.to_string(),
            },
            direction: dir,
        };

        if let Some(orders) = &mut self.structured_query.order_by {
            orders.push(order);
        } else {
            self.structured_query.order_by = Some(vec![order]);
        }

        Ok(self)
    }

    /// Limits the number of results.
    pub fn limit(mut self, limit: i32) -> Self {
        self.structured_query.limit = Some(limit);
        self
    }

    /// Skips the first n results.
    pub fn offset(mut self, offset: i32) -> Self {
        self.structured_query.offset = Some(offset);
        self
    }

    /// Selects specific fields to return.
    pub fn select(mut self, fields: &[&str]) -> Self {
        let refs = fields.iter().map(|f| FieldReference { field_path: f.to_string() }).collect();
        self.structured_query.select = Some(Projection {
             fields: Some(refs),
        });
        self
    }
}

/// Helper to execute queries
pub async fn run_query_request(
    client: &ClientWithMiddleware,
    base_url: &str,
    parent: &str,
    structured_query: StructuredQuery,
) -> Result<Vec<Document>, FirestoreError> {
    let url = format!("{}:runQuery", base_url);

    // Parent should be: projects/{project_id}/databases/{database_id}/documents/...

    let request = RunQueryRequest {
        parent: Some(parent.to_string()),
        structured_query: Some(structured_query),
        transaction: None,
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(FirestoreError::ApiError(format!(
            "Run query failed {}: {}",
            status, text
        )));
    }

    let responses: Vec<RunQueryResponse> = response.json().await?;

    let mut documents = Vec::new();
    for res in responses {
        if let Some(doc) = res.document {
            documents.push(doc);
        }
    }

    Ok(documents)
}
