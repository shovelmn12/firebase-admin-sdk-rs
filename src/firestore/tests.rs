use super::*;
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use httpmock::prelude::*;
use serde_json::json;

#[tokio::test]
async fn test_list_root_collections() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let db = FirebaseFirestore::new_with_client(client, server.url("/v1/projects/test-project/databases/(default)/documents"));

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/projects/test-project/databases/(default)/documents:listCollectionIds")
            .header("content-type", "application/json")
            .json_body(json!({ "pageSize": 100 }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "collectionIds": ["users", "posts"],
                "nextPageToken": ""
            }));
    });

    let collections = db.list_collections().await.unwrap();
    assert_eq!(collections.len(), 2);
    assert_eq!(collections[0].path, format!("{}/users", db.base_url));
    assert_eq!(collections[1].path, format!("{}/posts", db.base_url));
    
    mock.assert();
}

#[tokio::test]
async fn test_document_list_collections() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let db = FirebaseFirestore::new_with_client(client, server.url("/v1/projects/test-project/databases/(default)/documents"));
    
    let doc_ref = db.doc("users/user1");

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/projects/test-project/databases/(default)/documents/users/user1:listCollectionIds")
            .header("content-type", "application/json")
            .json_body(json!({ "pageSize": 100 }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "collectionIds": ["orders", "messages"],
                "nextPageToken": ""
            }));
    });

    let collections = doc_ref.list_collections().await.unwrap();
    assert_eq!(collections.len(), 2);
    assert_eq!(collections[0].path, format!("{}/orders", doc_ref.path));
    assert_eq!(collections[1].path, format!("{}/messages", doc_ref.path));
    
    mock.assert();
}

#[tokio::test]
async fn test_run_transaction() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let db = FirebaseFirestore::new_with_client(client, server.url("/v1/projects/test-project/databases/(default)/documents"));

    let transaction_id = "test-transaction-id";

    // 1. Mock beginTransaction
    let begin_mock = server.mock(|when, then| {
        when.method(POST)
            .path_matches(".*:beginTransaction");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({ "transaction": transaction_id }));
    });

    // 2. Mock Get (within transaction)
    let get_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/projects/test-project/databases/(default)/documents/users/user1")
            .query_param("transaction", transaction_id);
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "projects/test-project/databases/(default)/documents/users/user1",
                "fields": { "counter": { "integerValue": "10" } },
                "createTime": "2023-01-01T00:00:00Z",
                "updateTime": "2023-01-01T00:00:00Z"
            }));
    });

    // 3. Mock Commit
    let commit_mock = server.mock(|when, then| {
        when.method(POST)
            .path_matches(".*:commit");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({ "commitTime": "2023-01-01T00:00:00Z" }));
    });

    let result = db.run_transaction(|transaction| {
        async move {
            let snapshot: Option<serde_json::Value> = transaction.get("users/user1").await?;
            let counter = snapshot.unwrap().get("counter").and_then(|v| v.as_i64()).unwrap();
            
            transaction.update("users/user1", &json!({ "counter": counter + 1 }))?;
            
            Ok(counter)
        }
    }).await.unwrap();

    assert_eq!(result, 10);
    
    begin_mock.assert();
    get_mock.assert();
    commit_mock.assert();
}

#[tokio::test]
async fn test_get_document() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let db = FirebaseFirestore::new_with_client(client, server.url("/v1/projects/test-project/databases/(default)/documents"));

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/projects/test-project/databases/(default)/documents/users/user1");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "projects/test-project/databases/(default)/documents/users/user1",
                "fields": { "name": { "stringValue": "John Doe" } },
                "createTime": "2023-01-01T00:00:00Z",
                "updateTime": "2023-01-01T00:00:00Z"
            }));
    });

    let doc = db.doc("users/user1").get().await.unwrap();
    assert_eq!(doc.id, "user1");
    let fields = doc.document.unwrap().fields;
    if let Some(crate::firestore::models::ValueType::StringValue(s)) = fields.get("name").map(|v| &v.value_type) {
        assert_eq!(s, "John Doe");
    } else {
        panic!("Field 'name' mismatch");
    }
    
    mock.assert();
}

#[tokio::test]
async fn test_set_document() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let db = FirebaseFirestore::new_with_client(client, server.url("/v1/projects/test-project/databases/(default)/documents"));

    let mock = server.mock(|when, then| {
        when.method(PATCH)
            .path("/v1/projects/test-project/databases/(default)/documents/users/user1")
            .json_body(json!({
                "fields": {
                    "name": { "stringValue": "John Doe" }
                }
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "projects/test-project/databases/(default)/documents/users/user1",
                "fields": { "name": { "stringValue": "John Doe" } },
                "createTime": "2023-01-01T00:00:00Z",
                "updateTime": "2023-01-01T00:00:00Z"
            }));
    });

    #[derive(serde::Serialize)]
    struct User {
        name: String,
    }

    let result = db.doc("users/user1").set(&User { name: "John Doe".to_string() }).await.unwrap();
    assert!(!result.write_time.is_empty());
    
    mock.assert();
}

#[tokio::test]
async fn test_update_document() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let db = FirebaseFirestore::new_with_client(client, server.url("/v1/projects/test-project/databases/(default)/documents"));

    let mock = server.mock(|when, then| {
        when.method(PATCH)
            .path("/v1/projects/test-project/databases/(default)/documents/users/user1")
            .query_param("updateMask.fieldPaths", "name")
            .json_body(json!({
                "fields": {
                    "name": { "stringValue": "Jane Doe" }
                }
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "projects/test-project/databases/(default)/documents/users/user1",
                "fields": { "name": { "stringValue": "Jane Doe" } },
                "createTime": "2023-01-01T00:00:00Z",
                "updateTime": "2023-01-02T00:00:00Z"
            }));
    });

    #[derive(serde::Serialize)]
    struct UserUpdate {
        name: String,
    }

    let result = db.doc("users/user1")
        .update(&UserUpdate { name: "Jane Doe".to_string() }, Some(vec!["name".to_string()]))
        .await
        .unwrap();
    assert!(!result.write_time.is_empty());
    
    mock.assert();
}

#[tokio::test]
async fn test_delete_document() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let db = FirebaseFirestore::new_with_client(client, server.url("/v1/projects/test-project/databases/(default)/documents"));

    let mock = server.mock(|when, then| {
        when.method(DELETE)
            .path("/v1/projects/test-project/databases/(default)/documents/users/user1");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({}));
    });

    let result = db.doc("users/user1").delete().await.unwrap();
    assert!(!result.write_time.is_empty());
    
    mock.assert();
}

#[tokio::test]
async fn test_collection_add() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let db = FirebaseFirestore::new_with_client(client, server.url("/v1/projects/test-project/databases/(default)/documents"));

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/projects/test-project/databases/(default)/documents/users")
            .json_body(json!({
                "fields": {
                    "name": { "stringValue": "New User" }
                }
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "projects/test-project/databases/(default)/documents/users/auto-id",
                "fields": { "name": { "stringValue": "New User" } },
                "createTime": "2023-01-01T00:00:00Z",
                "updateTime": "2023-01-01T00:00:00Z"
            }));
    });

    #[derive(serde::Serialize)]
    struct NewUser {
        name: String,
    }

    let doc_ref = db.collection("users").add(&NewUser { name: "New User".to_string() }).await.unwrap();
    assert_eq!(doc_ref.path, "projects/test-project/databases/(default)/documents/users/auto-id");
    
    mock.assert();
}