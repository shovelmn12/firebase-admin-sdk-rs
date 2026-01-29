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


/// A definition of a Firestore query, including the target collection and filters.
///
/// This struct allows you to build a query independently of a specific Firestore client
/// or execution context, enabling reuse across different contexts.
#[derive(Clone, Debug)]
pub struct Query {
    pub(crate) collection_id: String,
    pub(crate) query: StructuredQuery,
}

impl Query {
    /// Creates a new `Query` targeting the specified collection.
    pub fn new(collection_id: impl Into<String>) -> Self {
        let collection_id = collection_id.into();
        Self {
            collection_id: collection_id.clone(),
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

    /// Adds a filter to the query.
    pub fn where_filter<T: Serialize>(
        mut self,
        field: &str,
        op: FieldOperator,
        value: T,
    ) -> Result<Self, FirestoreError> {
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

        if let Some(existing_where) = &self.query.where_clause {
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

            self.query.where_clause = Some(QueryFilter {
                filter_type: Some(FilterType::CompositeFilter(new_composite)),
            });
        } else {
            self.query.where_clause = Some(filter);
        }

        Ok(self)
    }

    /// Sorts the query results by the specified field.
    pub fn order_by(mut self, field: &str, direction: Direction) -> Self {
        let order = Order {
            field: FieldReference {
                field_path: field.to_string(),
            },
            direction,
        };

        if let Some(order_by) = &mut self.query.order_by {
            order_by.push(order);
        } else {
            self.query.order_by = Some(vec![order]);
        }

        self
    }

    /// Limits the number of documents returned.
    pub fn limit(mut self, limit: i32) -> Self {
        self.query.limit = Some(limit);
        self
    }

    /// Skips the first N documents.
    pub fn offset(mut self, offset: i32) -> Self {
        self.query.offset = Some(offset);
        self
    }
}

/// A `Query` attached to a Firestore client, ready for execution.
#[derive(Clone)]
pub struct ExecutableQuery<'a> {
    pub(crate) client: &'a ClientWithMiddleware,
    pub(crate) parent_path: String,
    pub(crate) query: Query,
}

impl<'a> ExecutableQuery<'a> {
    pub(crate) fn new(
        client: &'a ClientWithMiddleware,
        parent_path: String,
        query: Query,
    ) -> Self {
        Self {
            client,
            parent_path,
            query,
        }
    }

    // Proxy methods to modify the underlying Query (builder pattern on ExecutableQuery)

    /// Adds a filter to the query.
    pub fn where_filter<T: Serialize>(
        self,
        field: &str,
        op: FieldOperator,
        value: T,
    ) -> Result<Self, FirestoreError> {
        Ok(Self {
            query: self.query.where_filter(field, op, value)?,
            ..self
        })
    }

    /// Sorts the query results.
    pub fn order_by(self, field: &str, direction: Direction) -> Self {
        Self {
            query: self.query.order_by(field, direction),
            ..self
        }
    }

    /// Limits the results.
    pub fn limit(self, limit: i32) -> Self {
        Self {
            query: self.query.limit(limit),
            ..self
        }
    }

    /// offsets the results.
    pub fn offset(self, offset: i32) -> Self {
        Self {
            query: self.query.offset(offset),
            ..self
        }
    }

    /// Executes the query and returns the results as a `QuerySnapshot`.
    pub async fn get(&self) -> Result<QuerySnapshot<'a>, FirestoreError> {
        let url = format!("{}:runQuery", self.parent_path);

        let request = RunQueryRequest {
            parent: self.parent_path.clone(),
            structured_query: Some(self.query.query.clone()),
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

        let responses: Vec<RunQueryResponse> = response.json().await?;

        let mut documents = Vec::new();
        let mut read_time = None;

        for res in responses {
            if let Some(rt) = res.read_time {
                read_time = Some(rt);
            }

            if let Some(doc) = res.document {
                let name = doc.name.clone();
                let id = name.split('/').last().unwrap_or_default().to_string();

                let doc_ref = DocumentReference {
                    client: self.client,
                    path: name,
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
            structured_query: Some(self.query.query.clone()),
        };

        let target = Target {
            target_type: Some(TargetType::Query(query_target)),
            target_id: Some(1),
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
