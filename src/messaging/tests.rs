use super::*;
use crate::messaging::models::*;
use serde_json::json;

#[test]
fn test_message_serialization_basic() {
    let message = Message {
        token: Some("token123".to_string()),
        notification: Some(Notification {
            title: Some("Title".to_string()),
            body: Some("Body".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let json = serde_json::to_value(&message).unwrap();
    assert_eq!(json["token"], "token123");
    assert_eq!(json["notification"]["title"], "Title");
    assert_eq!(json["notification"]["body"], "Body");
}

#[test]
fn test_message_serialization_android() {
    let message = Message {
        topic: Some("weather".to_string()),
        android: Some(AndroidConfig {
            priority: Some(AndroidMessagePriority::High),
            notification: Some(AndroidNotification {
                icon: Some("stock_ticker_update".to_string()),
                color: Some("#f45342".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let json = serde_json::to_value(&message).unwrap();
    assert_eq!(json["topic"], "weather");
    assert_eq!(json["android"]["priority"], "HIGH");
    assert_eq!(json["android"]["notification"]["icon"], "stock_ticker_update");
    assert_eq!(json["android"]["notification"]["color"], "#f45342");
}

#[test]
fn test_message_serialization_apns() {
    let message = Message {
        token: Some("token123".to_string()),
        apns: Some(ApnsConfig {
            headers: Some(std::collections::HashMap::from([("apns-priority".to_string(), "10".to_string())])),
            payload: Some(ApnsPayload {
                aps: Some(Aps {
                    alert: Some(ApsAlert::Dictionary(ApsAlertDictionary {
                        title: Some("Game Request".to_string()),
                        body: Some("Bob wants to play poker".to_string()),
                        action_loc_key: Some("PLAY".to_string()),
                        ..Default::default()
                    })),
                    badge: Some(1),
                    ..Default::default()
                }),
                custom_data: Some(std::collections::HashMap::from([("acme1".to_string(), json!("bar"))])),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let json = serde_json::to_value(&message).unwrap();
    assert_eq!(json["apns"]["headers"]["apns-priority"], "10");
    assert_eq!(json["apns"]["payload"]["aps"]["alert"]["title"], "Game Request");
    assert_eq!(json["apns"]["payload"]["acme1"], "bar");
}

#[test]
fn test_message_serialization_webpush() {
    let message = Message {
        token: Some("token123".to_string()),
        webpush: Some(WebpushConfig {
            notification: Some(json!({
                "title": "Fish",
                "body": "Bass",
                "icon": "main-icon.png"
            })),
            fcm_options: Some(WebpushFcmOptions {
                link: Some("https://example.com".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let json = serde_json::to_value(&message).unwrap();
    assert_eq!(json["webpush"]["notification"]["title"], "Fish");
    // Fixed the key to camelCase "fcmOptions"
    assert_eq!(json["webpush"]["fcmOptions"]["link"], "https://example.com");
}

#[test]
fn test_topic_management_serialization() {
    let request = TopicManagementRequest {
        to: "/topics/news".to_string(),
        registration_tokens: &["token1", "token2"],
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["to"], "/topics/news");
    let tokens = json["registration_tokens"].as_array().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], "token1");
}

#[test]
fn test_topic_management_response_deserialization() {
    let json = json!({
        "results": [
            {},
            {"error": "NOT_FOUND"},
            {}
        ]
    });

    let response: TopicManagementApiResponse = serde_json::from_value(json).unwrap();
    let results = response.results.unwrap();
    assert_eq!(results.len(), 3);
    assert!(results[0].error.is_none());
    assert_eq!(results[1].error.as_deref(), Some("NOT_FOUND"));
}

#[test]
fn test_batch_response_aggregation() {
    // Since we can't easily mock the async calls in unit tests without traits/mocks,
    // we will verify the BatchResponse struct logic indirectly by constructing it.
    let responses = vec![
        SendResponse { success: true, message_id: Some("id1".to_string()), error: None },
        SendResponse { success: false, message_id: None, error: Some("Fail".to_string()) },
        SendResponse { success: true, message_id: Some("id2".to_string()), error: None },
    ];

    let success_count = responses.iter().filter(|r| r.success).count();
    let failure_count = responses.len() - success_count;

    let batch = BatchResponse {
        success_count,
        failure_count,
        responses
    };

    assert_eq!(batch.success_count, 2);
    assert_eq!(batch.failure_count, 1);
    assert_eq!(batch.responses.len(), 3);
}

#[tokio::test]
async fn test_send_validation() {
    let sa_key = yup_oauth2::ServiceAccountKey {
        key_type: Some("service_account".to_string()),
        project_id: Some("test-project".to_string()),
        private_key: "-----BEGIN PRIVATE KEY-----\n-----END PRIVATE KEY-----\n".to_string(),
        client_email: "test@example.com".to_string(),
        client_id: Some("12345".to_string()),
        auth_uri: Some("https://accounts.google.com/o/oauth2/auth".to_string()),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        auth_provider_x509_cert_url: Some("https://www.googleapis.com/oauth2/v1/certs".to_string()),
        client_x509_cert_url: Some("https://www.googleapis.com/robot/v1/metadata/x509/test".to_string()),
        private_key_id: None,
    };
    let messaging = FirebaseMessaging::new(sa_key);

    // No target
    let msg1 = Message::default();
    let err = messaging.send(&msg1).await.unwrap_err();
    assert!(matches!(err, MessagingError::ApiError(_)));
    assert!(err.to_string().contains("exactly one of"));

    // Multiple targets
    let msg2 = Message {
        token: Some("token".to_string()),
        topic: Some("topic".to_string()),
        ..Default::default()
    };
    let err = messaging.send(&msg2).await.unwrap_err();
    assert!(matches!(err, MessagingError::ApiError(_)));
    assert!(err.to_string().contains("exactly one of"));

    // Multicast with target in base message
    let base_msg = Message {
        topic: Some("topic".to_string()),
        ..Default::default()
    };
    let err = messaging.send_multicast_request(&base_msg, &["token"], false).await.unwrap_err();
    assert!(matches!(err, MessagingError::ApiError(_)));
    assert!(err.to_string().contains("Multicast base message must not"));
}


#[test]
fn test_parse_multipart_response() {
    let sa_key = yup_oauth2::ServiceAccountKey {
        key_type: Some("service_account".to_string()),
        project_id: Some("test-project".to_string()),
        private_key: "-----BEGIN PRIVATE KEY-----\n-----END PRIVATE KEY-----\n".to_string(),
        client_email: "test@example.com".to_string(),
        client_id: Some("12345".to_string()),
        auth_uri: Some("https://accounts.google.com/o/oauth2/auth".to_string()),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        auth_provider_x509_cert_url: Some("https://www.googleapis.com/oauth2/v1/certs".to_string()),
        client_x509_cert_url: Some("https://www.googleapis.com/robot/v1/metadata/x509/test".to_string()),
        private_key_id: None,
    };
    let messaging = FirebaseMessaging::new(sa_key);

    let body = "--batch_123\r\n\
                Content-Type: application/http\r\n\
                Content-Transfer-Encoding: binary\r\n\
                \r\n\
                HTTP/1.1 200 OK\r\n\
                Content-Type: application/json; charset=UTF-8\r\n\
                \r\n\
                {\r\n\
                \x20 \"name\": \"projects/test-project/messages/1\"\r\n\
                }\r\n\
                --batch_123\r\n\
                Content-Type: application/http\r\n\
                Content-Transfer-Encoding: binary\r\n\
                \r\n\
                HTTP/1.1 400 Bad Request\r\n\
                Content-Type: application/json; charset=UTF-8\r\n\
                \r\n\
                {\r\n\
                \x20 \"error\": {\r\n\
                \x20   \"code\": 400,\r\n\
                \x20   \"message\": \"Invalid registration token\",\r\n\
                \x20   \"status\": \"INVALID_ARGUMENT\"\r\n\
                \x20 }\r\n\
                }\r\n\
                --batch_123--\r\n";

    let responses = messaging.parse_multipart_response(body, "batch_123").unwrap();
    assert_eq!(responses.len(), 2);
    assert!(responses[0].success);
    assert_eq!(responses[0].message_id.as_deref(), Some("projects/test-project/messages/1"));
    assert!(!responses[1].success);
    assert!(responses[1].error.is_some());
}

#[test]
fn test_message_serialization_condition() {
    let message = Message {
        condition: Some("'foo' in topics && 'bar' in topics".to_string()),
        notification: Some(Notification {
            title: Some("Title".to_string()),
            body: Some("Body".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let json = serde_json::to_value(&message).unwrap();
    assert_eq!(json["condition"], "'foo' in topics && 'bar' in topics");
    assert_eq!(json["notification"]["title"], "Title");
    assert_eq!(json["notification"]["body"], "Body");
}
