use super::*;
use crate::remote_config::models::{
    RemoteConfig, RemoteConfigParameter, RemoteConfigParameterValue,
};
use httpmock::prelude::*;
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use std::collections::HashMap;

#[tokio::test]
async fn test_get_remote_config() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let base_url = server.url("/v1/projects/test-project/remoteConfig");

    let rc = FirebaseRemoteConfig::new_with_client(client, base_url);

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/projects/test-project/remoteConfig");
        then.status(200)
            .header("content-type", "application/json")
            .header("ETag", "\"etag-123\"")
            .json_body(serde_json::json!({
                "parameters": {
                    "welcome_message": {
                        "defaultValue": {
                            "value": "Hello World"
                        }
                    }
                }
            }));
    });

    let config = rc.get().await.unwrap();
    assert_eq!(config.etag, "\"etag-123\"");
    assert!(config.parameters.contains_key("welcome_message"));

    let param_value = config
        .parameters
        .get("welcome_message")
        .unwrap()
        .default_value
        .as_ref()
        .unwrap();
    if let RemoteConfigParameterValue::Value { value } = param_value {
        assert_eq!(value, "Hello World");
    } else {
        panic!("Expected Value variant");
    }

    mock.assert();
}

#[tokio::test]
async fn test_publish_remote_config() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let base_url = server.url("/v1/projects/test-project/remoteConfig");

    let rc = FirebaseRemoteConfig::new_with_client(client, base_url);

    let mut parameters = HashMap::new();
    parameters.insert(
        "welcome_message".to_string(),
        RemoteConfigParameter {
            default_value: Some(RemoteConfigParameterValue::Value {
                value: "Welcome!".to_string(),
            }),
            conditional_values: HashMap::new(),
            description: Some("Welcome message".to_string()),
        },
    );

    let config_to_publish = RemoteConfig {
        parameters,
        etag: "\"old-etag\"".to_string(),
        conditions: vec![],
        parameter_groups: HashMap::new(),
        version: None,
    };

    let mock = server.mock(|when, then| {
        when.method(PUT)
            .path("/v1/projects/test-project/remoteConfig")
            .header("If-Match", "\"old-etag\"")
            .json_body(serde_json::json!({
                "parameters": {
                    "welcome_message": {
                        "defaultValue": {
                            "value": "Welcome!"
                        },
                        "conditionalValues": {},
                        "description": "Welcome message"
                    }
                },
                "conditions": [],
                "parameterGroups": {},
                "version": null
            }));
        then.status(200)
            .header("content-type", "application/json")
            .header("ETag", "\"new-etag\"")
            .json_body(serde_json::json!({
                "parameters": {
                    "welcome_message": {
                        "defaultValue": {
                            "value": "Welcome!"
                        }
                    }
                },
                "etag": "\"new-etag\""
            }));
    });

    let published_config = rc.publish(config_to_publish).await.unwrap();
    assert_eq!(published_config.etag, "\"new-etag\"");

    mock.assert();
}
