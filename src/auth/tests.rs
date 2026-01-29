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

#[tokio::test]
async fn test_create_oidc_provider_config() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let project_config = ProjectConfig::new_with_client(client, server.url("/v2/projects/test-project"));

    let request = crate::auth::project_config::CreateOidcProviderConfigRequest {
        oauth_idp_config_id: "oidc.test".to_string(),
        display_name: Some("Test OIDC".to_string()),
        enabled: Some(true),
        client_id: "client-id".to_string(),
        issuer: "https://issuer.com".to_string(),
        ..Default::default()
    };

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v2/projects/test-project/oauthIdpConfigs")
            .query_param("oauthIdpConfigId", "oidc.test")
            .json_body(json!({
                "displayName": "Test OIDC",
                "enabled": true,
                "clientId": "client-id",
                "issuer": "https://issuer.com"
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "projects/test-project/oauthIdpConfigs/oidc.test",
                "displayName": "Test OIDC",
                "enabled": true,
                "clientId": "client-id",
                "issuer": "https://issuer.com"
            }));
    });

    let config = project_config.create_oidc_provider_config(request).await.unwrap();
    assert_eq!(config.name, "projects/test-project/oauthIdpConfigs/oidc.test");
    
    mock.assert();
}

#[tokio::test]
async fn test_create_saml_provider_config() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let project_config = ProjectConfig::new_with_client(client, server.url("/v2/projects/test-project"));

    let request = crate::auth::project_config::CreateSamlProviderConfigRequest {
        inbound_saml_config_id: "saml.test".to_string(),
        display_name: Some("Test SAML".to_string()),
        enabled: Some(true),
        idp_config: crate::auth::project_config::SamlIdpConfig {
            idp_entity_id: Some("idp-entity".to_string()),
            sso_url: Some("https://sso.com".to_string()),
            ..Default::default()
        },
        sp_config: crate::auth::project_config::SamlSpConfig {
            sp_entity_id: Some("sp-entity".to_string()),
            ..Default::default()
        },
    };

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v2/projects/test-project/inboundSamlConfigs")
            .query_param("inboundSamlConfigId", "saml.test")
            .json_body(json!({
                "displayName": "Test SAML",
                "enabled": true,
                "idpConfig": {
                    "idpEntityId": "idp-entity",
                    "ssoUrl": "https://sso.com"
                },
                "spConfig": {
                    "spEntityId": "sp-entity"
                }
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "name": "projects/test-project/inboundSamlConfigs/saml.test",
                "displayName": "Test SAML",
                "enabled": true,
                "idpConfig": {
                    "idpEntityId": "idp-entity",
                    "ssoUrl": "https://sso.com"
                },
                "spConfig": {
                    "spEntityId": "sp-entity"
                }
            }));
    });

    let config = project_config.create_saml_provider_config(request).await.unwrap();
    assert_eq!(config.name, "projects/test-project/inboundSamlConfigs/saml.test");
    
    mock.assert();
}

#[tokio::test]
async fn test_auth_error_parsing() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let auth = FirebaseAuth::new_with_client(client, server.url("/v1/projects/test-project"));

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/projects/test-project/accounts");
        then.status(400)
            .header("content-type", "application/json")
            .json_body(json!({
                "error": {
                    "code": 400,
                    "message": "EMAIL_EXISTS",
                    "errors": [
                        {
                            "message": "EMAIL_EXISTS",
                            "domain": "global",
                            "reason": "invalid"
                        }
                    ]
                }
            }));
    });

    let request = crate::auth::models::CreateUserRequest {
        email: Some("exists@example.com".to_string()),
        ..Default::default()
    };

    let result = auth.create_user(request).await;
    assert!(result.is_err());
    if let Err(AuthError::ApiError(msg)) = result {
        assert_eq!(msg, "EMAIL_EXISTS (code: 400)");
    } else {
        panic!("Expected ApiError");
    }
    
    mock.assert();
}

#[tokio::test]
async fn test_create_user_success() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let auth = FirebaseAuth::new_with_client(client, server.url("/v1/projects/test-project"));

    let request = crate::auth::models::CreateUserRequest {
        email: Some("newuser@example.com".to_string()),
        password: Some("password123".to_string()),
        display_name: Some("New User".to_string()),
        ..Default::default()
    };

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/projects/test-project/accounts")
            .header("content-type", "application/json")
            .json_body(json!({
                "email": "newuser@example.com",
                "password": "password123",
                "displayName": "New User"
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "localId": "new-uid",
                "email": "newuser@example.com",
                "displayName": "New User",
                "emailVerified": false,
                "disabled": false
            }));
    });

    let user = auth.create_user(request).await.unwrap();
    assert_eq!(user.local_id, "new-uid");
    assert_eq!(user.email.unwrap(), "newuser@example.com");
    
    mock.assert();
}

#[tokio::test]
async fn test_update_user_success() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let auth = FirebaseAuth::new_with_client(client, server.url("/v1/projects/test-project"));

    let request = crate::auth::models::UpdateUserRequest {
        local_id: "user-uid".to_string(),
        display_name: Some("Updated Name".to_string()),
        ..Default::default()
    };

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/projects/test-project/accounts:update")
            .header("content-type", "application/json")
            .json_body(json!({
                "localId": "user-uid",
                "displayName": "Updated Name"
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "localId": "user-uid",
                "displayName": "Updated Name",
                "emailVerified": false,
                "disabled": false
            }));
    });

    let user = auth.update_user(request).await.unwrap();
    assert_eq!(user.local_id, "user-uid");
    assert_eq!(user.display_name.unwrap(), "Updated Name");
    
    mock.assert();
}

#[tokio::test]
async fn test_delete_user_success() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let auth = FirebaseAuth::new_with_client(client, server.url("/v1/projects/test-project"));

    let uid = "user-uid";

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/projects/test-project/accounts:delete")
            .header("content-type", "application/json")
            .json_body(json!({
                "localId": uid
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({}));
    });

    auth.delete_user(uid).await.unwrap();
    
    mock.assert();
}

#[tokio::test]
async fn test_get_user_by_email() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let auth = FirebaseAuth::new_with_client(client, server.url("/v1/projects/test-project"));

    let email = "test@example.com";

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/projects/test-project/accounts:lookup")
            .header("content-type", "application/json")
            .json_body(json!({
                "email": [email]
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "users": [
                    {
                        "localId": "test-uid",
                        "email": email,
                        "emailVerified": true,
                        "disabled": false
                    }
                ]
            }));
    });

    let user = auth.get_user_by_email(email).await.unwrap();
    assert_eq!(user.local_id, "test-uid");
    assert_eq!(user.email.unwrap(), email);
    
    mock.assert();
}

#[tokio::test]
async fn test_list_users() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    let auth = FirebaseAuth::new_with_client(client, server.url("/v1/projects/test-project"));

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/projects/test-project/accounts")
            .query_param("maxResults", "100");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "users": [
                    {
                        "localId": "user1",
                        "email": "user1@example.com",
                        "emailVerified": false,
                        "disabled": false
                    },
                    {
                        "localId": "user2",
                        "email": "user2@example.com",
                        "emailVerified": true,
                        "disabled": false
                    }
                ],
                "nextPageToken": "next-token"
            }));
    });

    let result = auth.list_users(100, None).await.unwrap();
    let users = result.users.unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].local_id, "user1");
    assert_eq!(result.next_page_token.unwrap(), "next-token");
    
    mock.assert();
}