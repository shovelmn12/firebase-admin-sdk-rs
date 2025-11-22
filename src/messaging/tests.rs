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
