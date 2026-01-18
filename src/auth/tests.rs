use super::*;
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use httpmock::prelude::*;
use serde_json::json;
use crate::auth::models::{ActionCodeSettings, AndroidSettings, IosSettings};

#[tokio::test]
async fn test_generate_password_reset_link() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let auth = FirebaseAuth::new_with_client(client, server.url("/v1/projects/test-project"));

    let email = "test@example.com";
    let settings = ActionCodeSettings {
        url: "https://www.example.com/finishSignUp".to_string(),
        handle_code_in_app: Some(true),
        ios: Some(IosSettings {
            bundle_id: "com.example.ios".to_string(),
        }),
        android: Some(AndroidSettings {
            package_name: "com.example.android".to_string(),
            install_app: Some(true),
            minimum_version: Some("12".to_string()),
        }),
        dynamic_link_domain: Some("example.page.link".to_string()),
    };

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/projects/test-project/accounts:sendOobCode")
            .header("content-type", "application/json")
            .json_body(json!({
                "requestType": "PASSWORD_RESET",
                "email": email,
                "continueUrl": "https://www.example.com/finishSignUp",
                "canHandleCodeInApp": true,
                "dynamicLinkDomain": "example.page.link",
                "iOSBundleId": "com.example.ios",
                "androidPackageName": "com.example.android",
                "androidInstallApp": true,
                "androidMinimumVersion": "12"
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "email": email,
                "oobLink": "https://example.com/action?mode=resetPassword&oobCode=code"
            }));
    });

    let link = auth.generate_password_reset_link(email, Some(settings)).await.unwrap();
    assert_eq!(link, "https://example.com/action?mode=resetPassword&oobCode=code");
    
    mock.assert();
}

#[tokio::test]
async fn test_generate_email_verification_link() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let auth = FirebaseAuth::new_with_client(client, server.url("/v1/projects/test-project"));

    let email = "test@example.com";

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/projects/test-project/accounts:sendOobCode")
            .header("content-type", "application/json")
            .json_body(json!({
                "requestType": "VERIFY_EMAIL",
                "email": email
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "email": email,
                "oobLink": "https://example.com/action?mode=verifyEmail&oobCode=code"
            }));
    });

    let link = auth.generate_email_verification_link(email, None).await.unwrap();
    assert_eq!(link, "https://example.com/action?mode=verifyEmail&oobCode=code");
    
    mock.assert();
}
