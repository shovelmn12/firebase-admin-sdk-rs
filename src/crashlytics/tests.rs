use super::*;
use crate::FirebaseApp;
use yup_oauth2::ServiceAccountKey;

fn create_mock_app() -> FirebaseApp {
    let key = ServiceAccountKey {
        key_type: Some("service_account".to_string()),
        project_id: Some("test-project".to_string()),
        private_key_id: Some("test_key_id".to_string()),
        private_key: "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQD \n-----END PRIVATE KEY-----".to_string(),
        client_email: "test@test-project.iam.gserviceaccount.com".to_string(),
        client_id: Some("test_client_id".to_string()),
        auth_uri: Some("https://accounts.google.com/o/oauth2/auth".to_string()),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        auth_provider_x509_cert_url: Some("https://www.googleapis.com/oauth2/v1/certs".to_string()),
        client_x509_cert_url: Some("https://www.googleapis.com/robot/v1/metadata/x509/test%40test-project.iam.gserviceaccount.com".to_string()),
    };
    FirebaseApp::new(key)
}

#[tokio::test]
async fn test_crashlytics_instance() {
    let app = create_mock_app();
    let _crashlytics = app.crashlytics();
    // Basic sanity check that we can create the instance.
    // Actual calls would fail due to auth and hardcoded URLs.
}
