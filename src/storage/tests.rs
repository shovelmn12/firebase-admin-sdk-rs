use crate::storage::FirebaseStorage;
use httpmock::Method::{DELETE, GET, POST};
use httpmock::MockServer;
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use serde_json::json;

#[tokio::test]
async fn test_storage_flow() {
    // Start a mock server
    let server = MockServer::start();

    // Use a plain client without AuthMiddleware for testing
    let client = ClientBuilder::new(Client::new()).build();
    let base_url = server.url("");
    let project_id = "test-project".to_string();

    let storage = FirebaseStorage::new_with_client(client, base_url, project_id);

    let bucket_name = "test-project.appspot.com";
    let file_name = "test.txt";
    let encoded_file_name = "test.txt";

    // Mock Upload
    let upload_mock = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/upload/storage/v1/b/{}/o", bucket_name))
            .query_param("uploadType", "media")
            .query_param("name", file_name)
            .header("Content-Type", "text/plain");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "bucket": bucket_name,
                "name": file_name,
                "size": "11",
                "contentType": "text/plain"
            }));
    });

    // Mock Download (More specific, must be defined first if FIFO)
    let download_mock = server.mock(|when, then| {
        when.method(GET)
            .path(format!("/b/{}/o/{}", bucket_name, encoded_file_name))
            .query_param("alt", "media");
        then.status(200)
            .body("Hello World");
    });

    // Mock Metadata (Less specific, defined after)
    let metadata_mock = server.mock(|when, then| {
        when.method(GET)
            .path(format!("/b/{}/o/{}", bucket_name, encoded_file_name));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "bucket": bucket_name,
                "name": file_name,
                "size": "11",
                "contentType": "text/plain"
            }));
    });

    // Mock Delete
    let delete_mock = server.mock(|when, then| {
        when.method(DELETE)
            .path(format!("/b/{}/o/{}", bucket_name, encoded_file_name));
        then.status(204);
    });

    let bucket = storage.bucket(None);
    assert_eq!(bucket.name(), bucket_name);

    let file = bucket.file(file_name);
    assert_eq!(file.name(), file_name);

    // Test Upload
    file.save("Hello World", "text/plain").await.unwrap();
    upload_mock.assert();

    // Test Download
    let content = file.download().await.unwrap();
    assert_eq!(content, "Hello World".as_bytes());
    download_mock.assert();

    // Test Metadata
    let metadata = file.get_metadata().await.unwrap();
    assert_eq!(metadata.name.unwrap(), file_name);
    metadata_mock.assert();

    // Test Delete
    file.delete().await.unwrap();
    delete_mock.assert();
}
