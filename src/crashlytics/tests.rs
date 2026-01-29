use super::*;
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use httpmock::prelude::*;

#[tokio::test]
async fn test_delete_crash_reports() {
    let server = MockServer::start();
    let client = ClientBuilder::new(Client::new()).build();
    // We construct the base URL such that it matches the structure expected by delete_crash_reports:
    // {base_url}/apps/{app_id}/users/{user_id}/crashReports
    // The server mock will match against this path.
    let base_url = server.url("/v1alpha/projects/test-project");
    
    let crashlytics = FirebaseCrashlytics::new_with_client(client, base_url);

    let app_id = "1:1234567890:android:321abc456def7890";
    let user_id = "user123";

    let mock = server.mock(|when, then| {
        when.method(DELETE)
            .path(format!("/v1alpha/projects/test-project/apps/{}/users/{}/crashReports", app_id, user_id));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(serde_json::json!({}));
    });

    let result = crashlytics.delete_crash_reports(app_id, user_id).await;
    assert!(result.is_ok());
    
    mock.assert();
}