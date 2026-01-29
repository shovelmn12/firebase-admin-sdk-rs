use super::listen::{listen_request, ListenStream};
use super::models::{
    CollectionSelector, CompositeFilter, CompositeOperator, Direction, FieldFilter, FieldOperator,
    FieldReference, FilterType, ListenRequest, Order, QueryFilter, QueryTarget, RunQueryRequest,
    RunQueryResponse, StructuredQuery, Target, TargetType,
};
use super::reference::{
    convert_serde_value_to_firestore_value, extract_database_path, DocumentReference,
};
use super::snapshot::{DocumentSnapshot, QuerySnapshot};
use super::FirestoreError;
use reqwest::header;
use reqwest_middleware::ClientWithMiddleware;
use serde::Serialize;

/// A `Query` refers to a query which you can read or listen to.
///
/// You can also construct refined `Query` objects by adding filters and ordering.
#[derive(Clone)]
pub struct Query<'a> {
    pub(crate) client: &'a ClientWithMiddleware,
    pub(crate) parent_path: String, // projects/{id}/databases/{id}/documents or .../documents/col/doc
    pub(crate) query: StructuredQuery,
}

impl<'a> Query<'a> {
    pub(crate) fn new(
        client: &'a ClientWithMiddleware,
        parent_path: String,
        collection_id: String,
    ) -> Self {
        Self {
            client,
            parent_path,
            query: StructuredQuery {
                select: None,
                from: Some(vec![CollectionSelector {
                    collection_id,
                    all_descendants: None,
                }]),
                where_clause: None,
                order_by: None,
                start_at: None,
                end_at: None,
                offset: None,
                limit: None,
            },
        }
    }

    /// Creates and returns a new `Query` with the additional filter.
    ///
    /// # Arguments
    ///
    /// * `field` - The path of the field to filter (e.g., "age").
    /// * `op` - The operation to apply.
    /// * `value` - The value to compare against.
    pub fn where_filter<T: Serialize>(
        &self,
        field: &str,
        op: FieldOperator,
        value: T,
    ) -> Result<Query<'a>, FirestoreError> {
        let mut new_query = self.clone();

        let serde_value = serde_json::to_value(value)?;
        let firestore_value = convert_serde_value_to_firestore_value(serde_value)?;

        let filter = QueryFilter {
            filter_type: Some(FilterType::FieldFilter(FieldFilter {
                field: FieldReference {
                    field_path: field.to_string(),
                },
                op,
                value: firestore_value,
            })),
        };

        if let Some(existing_where) = &new_query.query.where_clause {
            // If there's already a filter, we need to combine them using AND.
            // If the existing filter is a CompositeFilter with AND, we can append.
            // Otherwise, we create a new CompositeFilter with AND.

            let new_composite = match &existing_where.filter_type {
                Some(FilterType::CompositeFilter(cf)) if cf.op == CompositeOperator::And => {
                    let mut filters = cf.filters.clone();
                    filters.push(filter);
                    CompositeFilter {
                        op: CompositeOperator::And,
                        filters,
                    }
                }
                _ => CompositeFilter {
                    op: CompositeOperator::And,
                    filters: vec![existing_where.clone(), filter],
                },
            };

            new_query.query.where_clause = Some(QueryFilter {
                filter_type: Some(FilterType::CompositeFilter(new_composite)),
            });
        } else {
            new_query.query.where_clause = Some(filter);
        }

        Ok(new_query)
    }

    /// Creates and returns a new `Query` that's additionally sorted by the specified field.
    pub fn order_by(&self, field: &str, direction: Direction) -> Query<'a> {
        let mut new_query = self.clone();

        let order = Order {
            field: FieldReference {
                field_path: field.to_string(),
            },
            direction,
        };

        if let Some(order_by) = &mut new_query.query.order_by {
            order_by.push(order);
        } else {
            new_query.query.order_by = Some(vec![order]);
        }

        new_query
    }

    /// Creates and returns a new `Query` that only returns the first matching documents.
    pub fn limit(&self, limit: i32) -> Query<'a> {
        let mut new_query = self.clone();
        new_query.query.limit = Some(limit);
        new_query
    }

    /// Creates and returns a new `Query` that skips the first matching documents.
    pub fn offset(&self, offset: i32) -> Query<'a> {
        let mut new_query = self.clone();
        new_query.query.offset = Some(offset);
        new_query
    }

    /// Executes the query and returns the results as a `QuerySnapshot`.
    pub async fn get(&self) -> Result<QuerySnapshot<'a>, FirestoreError> {
        let url = format!("{}:runQuery", self.parent_path);

        let request = RunQueryRequest {
            parent: self.parent_path.clone(),
            structured_query: Some(self.query.clone()),
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
                "Run query failed {}: {}",
                status, text
            )));
        }

        // The response is a stream of JSON objects, e.g. [{...}, {...}] or line-delimited?
        // The REST API usually returns a JSON array [ { "document": ... }, ... ] for runQuery?
        // Wait, documentation says "The response body contains a stream of RunQueryResponse messages."
        // In standard REST/JSON, this often means a JSON array.

        // Let's assume it returns a JSON array of RunQueryResponse objects.
        // We need to parse this. `response.json::<Vec<RunQueryResponse>>()` might work.

        let responses: Vec<RunQueryResponse> = response.json().await?;

        let mut documents = Vec::new();
        let mut read_time = None;

        for res in responses {
            if let Some(rt) = res.read_time {
                read_time = Some(rt);
            }

            if let Some(doc) = res.document {
                // Construct DocumentSnapshot
                // Extract ID from name
                let name = doc.name.clone();
                let id = name.split('/').last().unwrap_or_default().to_string();

                let doc_ref = DocumentReference {
                    client: self.client,
                    path: name, // The full path
                };

                documents.push(DocumentSnapshot {
                    id,
                    reference: doc_ref,
                    document: Some(doc),
                    read_time: read_time.clone(),
                });
            }
        }

        Ok(QuerySnapshot {
            documents,
            read_time,
        })
    }

    /// Listens to changes to the query results.
    pub async fn listen(&self) -> Result<ListenStream, FirestoreError> {
        let database = extract_database_path(&self.parent_path);

        let query_target = QueryTarget {
            parent: self.parent_path.clone(),
            structured_query: Some(self.query.clone()),
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
