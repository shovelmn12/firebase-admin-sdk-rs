use super::*;
use crate::core::middleware::AuthMiddleware;
use crate::storage::file::{GetSignedUrlOptions, SignedUrlMethod, ObjectMetadata};
use yup_oauth2::ServiceAccountKey;
use std::time::{SystemTime, Duration};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use httpmock::prelude::*;
use serde_json::json;

const PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDhKEj4y/U48B/5
dqaoSxCIm1uw1wTJDSUvgyjlyg8TFw1uhYt3wW9bKfvOG5a4tb+txSDaBV7buHVm
q4AkDXpZq7HP/h29ElJEwiKT9Gl8m3al4IeMUehD5EHChtTB55RtVzFI4m/vJLAR
nW9sGU6izp+S8AcQ2GjYAFbADUFiCxwkWYjBW95V+SYLVd8UKQcUJGR7tY/X7JZb
InBdT04Fii0k9hTpTDBAFiyJzmoj7GXORWLH9ejBZ0ulCjsqgt7ojevy9vjL5DcW
mRQf5SljDmy0uxw7wL4uCRLxGIMJ8FwuXTWuYaFh3BNW0vJGtTO8BKnyxsNaJOpm
2wQJ8IY3AgMBAAECggEAAawG6Vy6XsiJtD8z+vPzv3qdMlxREMfO4DdydPe3J5vN
jGXkJJOEfCzSTd7ZPliQf9Mtl0Y1mh7DNcFNm6GYqFR6EY1ViIiQ9n8VOqa0pymQ
YVL1hA6SUaQUSO7aDZvmokPk0yG7Vbn0BMLNMlmjF9po8ke4sGCrBqTvVVBujTJ8
W0mehX2JkVncXa4bFJcTr190f0RbBDDc0QnUSlJdQaPaitxwqFcklkWPJ90GLDl+
m8+R5srhYz9qcqYL5Q+8goHo2N7jqYE41T9SEEaPtm1/DcGPj5RAVLLENPHVy1DM
2VmqZTTx3qjMxoOQndHOXgw1PzxWBsgvULRhk5SWwQKBgQD1p0L7M65pEdvtlEzS
IPidXpqF2+1WwP870yZ8GwCW6y+jX7PFhcGG7m8/owSeQLRjejdoftXoaOiEd4ul
BWCKhkJw7uqKkrTubnAhWSFPsg+KTFUxGzh09mnZvi1fQ3zwoK52KJcd5uDrVGX5
46trDfcaCYAKvfgWvnO4C6dEGQKBgQDqpAbfYXXYCucDZwGjBxhr7WYrC1g0mAr7
jDQQ741b7C5BgQ9dAXRuXHJF7bUWRv0BpER8MvihPh8zgWYaeMqIgfyQstQKa+ts
h9DwLvC+hN/yOy/r7iHu8UIqn0ISVkULCTQkaWHLOnQW1g9xsmvgmnZv8NwmfNpd
XB0nitLmzwKBgBUP0TNee/6wNE4LYAbIIujDOrZtY80DYR7M/Mi5O/S0l3IHe49c
53ndKZaoMHYtEApTaTrBXS+/BuiMo2Fzs5JM7pdmNJ/K8k5bE6wYSz3dA24VG1zJ
e66zjeHIZ3V6gNTUwgCJfGNo7zHeG5wwQ/s6yEvoMp05KnMwwxUtkprJAoGBAJ4x
0nReiA4NY6z2kLLygLObTeutbV2gOJ9Z6myUpZCZDqKZOdtxtKcHav/cgN+xIrkt
oALAdsJ3WJ/oGQe18o7QXJDOEImqMwJsGyEj9KnuefIdl3SQi45GWF7WGry0Lz5+
iQoXhph3I3eWALmeGn9GhJ16HWNRgAO7q+hR/1kfAoGBAL5FVy2w6EdNJ4e60lSS
Ym4n/zE/bu7vZIka1dkoUOwqN0YoNfKA5L9zrv3NviF78qaHZHb6ODdcDbWB0ygz
1Lup8qmcMZ6mgxrf12LWpa0d5oR4UvSNUHuGFpItLbYTtpl72T899fNA+UPMhgEr
A0vhBaO9oh0OfLqzQjhjz3+j
-----END PRIVATE KEY-----";

fn create_dummy_middleware() -> AuthMiddleware {
    let key = ServiceAccountKey {
        key_type: Some("service_account".to_string()),
        project_id: Some("test-project".to_string()),
        private_key_id: Some("12345".to_string()),
        private_key: PRIVATE_KEY.to_string(),
        client_email: "test@test-project.iam.gserviceaccount.com".to_string(),
        client_id: Some("123".to_string()),
        auth_uri: None,
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        auth_provider_x509_cert_url: None,
        client_x509_cert_url: None,
    };
    AuthMiddleware::new(key)
}

#[tokio::test]
async fn test_set_metadata() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let middleware = create_dummy_middleware();
    
    // We point to the mock server, but FirebaseStorage expects base_url to be e.g. "https://storage.googleapis.com/storage/v1"
    // Our mock server URL is like "http://127.0.0.1:xxx".
    // File::set_metadata constructs URL as "{base_url}/b/{bucket}/o/{name}"
    // So if we set base_url to server.url(""), we get "{server_url}/b/..."
    
    let storage = FirebaseStorage::new_with_client(client, server.url(""), middleware);
    let bucket = storage.bucket(Some("test-bucket"));
    let file = bucket.file("test-file.txt");

    let metadata_update = ObjectMetadata {
        content_type: Some("application/json".to_string()),
        metadata: Some(std::collections::HashMap::from([("custom".to_string(), "value".to_string())])),
        ..Default::default()
    };

    let mock = server.mock(|when, then| {
        when.method(PATCH)
            .path("/b/test-bucket/o/test-file.txt")
            .header("content-type", "application/json")
            .json_body(json!({
                "contentType": "application/json",
                "metadata": {
                    "custom": "value"
                }
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "test-file.txt",
                "bucket": "test-bucket",
                "contentType": "application/json",
                "metadata": {
                    "custom": "value"
                }
            }));
    });

    let updated_metadata = file.set_metadata(&metadata_update).await.unwrap();
    
    assert_eq!(updated_metadata.content_type.unwrap(), "application/json");
    assert_eq!(updated_metadata.metadata.unwrap().get("custom").unwrap(), "value");
    
    mock.assert();
}

#[test]
fn test_get_signed_url() {
    let middleware = create_dummy_middleware();
    let storage = FirebaseStorage::new(middleware);
    let bucket = storage.bucket(Some("test-bucket"));
    let file = bucket.file("test-file.txt");

    let options = GetSignedUrlOptions {
        method: SignedUrlMethod::GET,
        expires: SystemTime::now() + Duration::from_secs(3600),
        content_type: None,
    };

    let url = file.get_signed_url(options).unwrap();
    
    // Basic validation
    assert!(url.starts_with("https://storage.googleapis.com/test-bucket/test-file.txt"));
    assert!(url.contains("X-Goog-Algorithm=GOOG4-RSA-SHA256"));
    assert!(url.contains("X-Goog-Credential="));
    assert!(url.contains("X-Goog-Signature="));
    
    // Check Credential format
    // test@test-project.iam.gserviceaccount.com/YYYYMMDD/auto/storage/goog4_request
    assert!(url.contains("test%40test-project.iam.gserviceaccount.com"));
}

#[tokio::test]
async fn test_save_file() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let middleware = create_dummy_middleware();
    // Using default base_url-like behavior but pointing to mock server
    let storage = FirebaseStorage::new_with_client(client, server.url(""), middleware);
    let bucket = storage.bucket(Some("test-bucket"));
    let file = bucket.file("test-file.txt");

    let content = "Hello, World!";

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/upload/storage/v1/b/test-bucket/o")
            .query_param("uploadType", "media")
            .query_param("name", "test-file.txt")
            .header("content-type", "text/plain")
            .body(content);
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "test-file.txt",
                "bucket": "test-bucket"
            }));
    });

    file.save(content, "text/plain").await.unwrap();
    
    mock.assert();
}

#[tokio::test]
async fn test_download_file() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let middleware = create_dummy_middleware();
    let storage = FirebaseStorage::new_with_client(client, server.url(""), middleware);
    let bucket = storage.bucket(Some("test-bucket"));
    let file = bucket.file("test-file.txt");

    let content = "Hello, World!";

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/b/test-bucket/o/test-file.txt")
            .query_param("alt", "media");
        then.status(200)
            .body(content);
    });

    let bytes = file.download().await.unwrap();
    assert_eq!(bytes, content.as_bytes());
    
    mock.assert();
}

#[tokio::test]
async fn test_delete_file() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let middleware = create_dummy_middleware();
    let storage = FirebaseStorage::new_with_client(client, server.url(""), middleware);
    let bucket = storage.bucket(Some("test-bucket"));
    let file = bucket.file("test-file.txt");

    let mock = server.mock(|when, then| {
        when.method(DELETE)
            .path("/b/test-bucket/o/test-file.txt");
        then.status(204);
    });

    file.delete().await.unwrap();
    
    mock.assert();
}

#[tokio::test]
async fn test_get_metadata() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let middleware = create_dummy_middleware();
    let storage = FirebaseStorage::new_with_client(client, server.url(""), middleware);
    let bucket = storage.bucket(Some("test-bucket"));
    let file = bucket.file("test-file.txt");

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/b/test-bucket/o/test-file.txt");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "test-file.txt",
                "bucket": "test-bucket",
                "size": "100"
            }));
    });

    let metadata = file.get_metadata().await.unwrap();
    assert_eq!(metadata.name.unwrap(), "test-file.txt");
    assert_eq!(metadata.size.unwrap(), "100");
    
    mock.assert();
}
