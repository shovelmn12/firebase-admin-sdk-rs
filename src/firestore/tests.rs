/*
#[cfg(test)]
mod tests {
    use crate::FirebaseApp;
    use httpmock::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use yup_oauth2::read_service_account_key;
    use httpmock::Method::PATCH;

    async fn create_test_app(server: &MockServer) -> FirebaseApp {
        let mut key = read_service_account_key("service_account.json")
            .await
            .unwrap();
        key.token_uri = server.url("/token");

        FirebaseApp::new(key)
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
    struct MyData {
        foo: String,
        bar: i64,
    }

    #[tokio::test]
    async fn test_get_document() {
        let server = MockServer::start();

        // Mock the OAuth2 token endpoint
        server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "access_token": "mock_token",
                    "token_type": "Bearer",
                    "expires_in": 3600
                }));
        });

        let firestore_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v1/projects/test-project/databases/(default)/documents/my-collection/my-doc");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "name": "projects/test-project/databases/(default)/documents/my-collection/my-doc",
                    "fields": {
                        "foo": { "stringValue": "Hello" },
                        "bar": { "integerValue": "123" },
                    },
                    "createTime": "2024-01-01T00:00:00Z",
                    "updateTime": "2024-01-01T00:00:00Z",
                }));
        });

        let app = create_test_app(&server).await;
        let firestore = app.firestore();
        let mut doc_ref = firestore.doc("my-collection/my-doc");
        doc_ref.path = server.url("/v1/projects/test-project/databases/(default)/documents/my-collection/my-doc");


        let data: MyData = doc_ref.get().await.unwrap().unwrap_or_default();

        assert_eq!(
            data,
            MyData {
                foo: "Hello".to_string(),
                bar: 123,
            }
        );
        firestore_mock.assert();
    }

    #[tokio::test]
    async fn test_add_document() {
        let server = MockServer::start();

        // Mock the OAuth2 token endpoint
        server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "access_token": "mock_token",
                    "token_type": "Bearer",
                    "expires_in": 3600
                }));
        });

        let firestore_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v1/projects/test-project/databases/(default)/documents/my-collection");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "name": "projects/test-project/databases/(default)/documents/my-collection/new-doc-id",
                    "fields": {
                        "foo": { "stringValue": "Hello" },
                        "bar": { "integerValue": "123" },
                    },
                    "createTime": "2024-01-01T00:00:00Z",
                    "updateTime": "2024-01-01T00:00:00Z",
                }));
        });

        let app = create_test_app(&server).await;
        let firestore = app.firestore();
        let col_ref = firestore.collection("my-collection");

        let data = MyData {
            foo: "Hello".to_string(),
            bar: 123,
        };

        let doc = col_ref.add(&data).await.unwrap();
        assert!(doc.name.ends_with("new-doc-id"));
        firestore_mock.assert();
    }

    #[tokio::test]
    async fn test_update_document() {
        let server = MockServer::start();

        // Mock the OAuth2 token endpoint
        server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "access_token": "mock_token",
                    "token_type": "Bearer",
                    "expires_in": 3600
                }));
        });

        let firestore_mock = server.mock(|when, then| {
            when.method(PATCH)
                .path("/v1/projects/test-project/databases/(default)/documents/my-collection/my-doc");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "name": "projects/test-project/databases/(default)/documents/my-collection/my-doc",
                    "fields": {
                        "foo": { "stringValue": "Updated" },
                        "bar": { "integerValue": "123" },
                    },
                    "createTime": "2024-01-01T00:00:00Z",
                    "updateTime": "2024-01-01T00:00:00Z",
                }));
        });

        let app = create_test_app(&server).await;
        let firestore = app.firestore();
        let mut doc_ref = firestore.doc("my-collection/my-doc");
        doc_ref.path = server.url("/v1/projects/test-project/databases/(default)/documents/my-collection/my-doc");


        let data = MyData {
            foo: "Updated".to_string(),
            bar: 123,
        };

        doc_ref.update(&data, None).await.unwrap();
        firestore_mock.assert();
    }
}
*/
