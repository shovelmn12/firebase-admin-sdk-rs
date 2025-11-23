#[cfg(test)]
mod tests {
    use crate::auth::models::CreateUserRequest;

    #[test]
    fn test_create_user_request_serialization() {
        let request = CreateUserRequest {
            email: Some("test@example.com".to_string()),
            password: Some("secret".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"email\":\"test@example.com\""));
        assert!(json.contains("\"password\":\"secret\""));
    }
}
