use super::*;
use httpmock::prelude::*;
use serde_json::json;
use crate::messaging::models::{Message, Notification};
use crate::core::middleware::AuthMiddleware;
use yup_oauth2::ServiceAccountKey;

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

#[tokio::test]
async fn test_send_message() {
    let server = MockServer::start();
    
    // 1. Mock OAuth2 Token
    let _token_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/token");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "access_token": "fake-token",
                "token_type": "Bearer",
                "expires_in": 3600
            }));
    });

    let key = ServiceAccountKey {
        key_type: Some("service_account".to_string()),
        project_id: Some("test-project".to_string()),
        private_key: PRIVATE_KEY.to_string(),
        client_email: "test@example.com".to_string(),
        token_uri: server.url("/token"),
        private_key_id: None,
        client_id: None,
        auth_uri: None,
        auth_provider_x509_cert_url: None,
        client_x509_cert_url: None,
    };
    let middleware = AuthMiddleware::new(key);
    
    let messaging = FirebaseMessaging::new_with_url(
        middleware, 
        server.url("/v1/projects/test-project/messages:send"),
        server.url("/batch"),
        server.url("/iid")
    );

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path_matches(".*");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "projects/test-project/messages/12345"
            }));
    });

    let message = Message {
        token: Some("test-token".to_string()),
        notification: Some(Notification {
            title: Some("Test Title".to_string()),
            body: Some("Test Body".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = messaging.send(&message, false).await.unwrap();
    assert_eq!(result, "projects/test-project/messages/12345");
    
    mock.assert();
}

#[tokio::test]
async fn test_subscribe_to_topic() {
    let server = MockServer::start();
    
    // 1. Mock OAuth2 Token
    let _token_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/token");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "access_token": "fake-token",
                "token_type": "Bearer",
                "expires_in": 3600
            }));
    });

    let key = ServiceAccountKey {
        key_type: Some("service_account".to_string()),
        project_id: Some("test-project".to_string()),
        private_key: PRIVATE_KEY.to_string(),
        client_email: "test@example.com".to_string(),
        token_uri: server.url("/token"),
        private_key_id: None,
        client_id: None,
        auth_uri: None,
        auth_provider_x509_cert_url: None,
        client_x509_cert_url: None,
    };
    let middleware = AuthMiddleware::new(key);
    
    let messaging = FirebaseMessaging::new_with_url(
        middleware,
        server.url("/v1/projects/test-project/messages:send"),
        server.url("/batch"),
        server.url("/iid"),
    );

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/iid/iid/v1:batchAdd")
            .json_body(json!({
                "to": "/topics/test-topic",
                "registration_tokens": ["token1", "token2"]
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "results": [
                    {},
                    { "error": "INVALID_ARGUMENT" }
                ]
            }));
    });

    let result = messaging.subscribe_to_topic(&["token1", "token2"], "test-topic").await.unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.failure_count, 1);
    assert_eq!(result.errors[0].reason, "INVALID_ARGUMENT");
    
    mock.assert();
}

#[tokio::test]
async fn test_unsubscribe_from_topic() {
    let server = MockServer::start();
    
    // 1. Mock OAuth2 Token
    let _token_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/token");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "access_token": "fake-token",
                "token_type": "Bearer",
                "expires_in": 3600
            }));
    });

    let key = ServiceAccountKey {
        key_type: Some("service_account".to_string()),
        project_id: Some("test-project".to_string()),
        private_key: PRIVATE_KEY.to_string(),
        client_email: "test@example.com".to_string(),
        token_uri: server.url("/token"),
        private_key_id: None,
        client_id: None,
        auth_uri: None,
        auth_provider_x509_cert_url: None,
        client_x509_cert_url: None,
    };
    let middleware = AuthMiddleware::new(key);
    
    let messaging = FirebaseMessaging::new_with_url(
        middleware,
        server.url("/v1/projects/test-project/messages:send"),
        server.url("/batch"),
        server.url("/iid"),
    );

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/iid/iid/v1:batchRemove")
            .json_body(json!({
                "to": "/topics/test-topic",
                "registration_tokens": ["token1"]
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "results": [
                    {}
                ]
            }));
    });

    let result = messaging.unsubscribe_from_topic(&["token1"], "test-topic").await.unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.failure_count, 0);
    
    mock.assert();
}