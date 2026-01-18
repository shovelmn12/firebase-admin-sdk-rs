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
