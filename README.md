# Firebase Admin SDK for Rust

A Rust implementation of the Firebase Admin SDK. This library allows you to interact with Firebase services such as Authentication, Cloud Messaging (FCM), Remote Config, and Firestore from your Rust backend.

## Features

-   **Authentication**: User management (create, update, delete, list, get), ID token verification, custom token creation, multi-tenancy support, and OIDC/SAML provider configuration.
-   **Cloud Messaging (FCM)**: Send messages (single, batch, multicast), manage topics, and support for all target types (token, topic, condition).
-   **Remote Config**: Get the active template, publish new templates (with ETag optimistic concurrency), rollback to previous versions, and list versions.
-   **Firestore**: Read and write documents, manage collections, perform atomic batch writes and transactions, and support real-time updates via the `listen()` API.
-   **Storage**: Upload, download, manage files, and generate V4 signed URLs in Google Cloud Storage buckets.
-   **Crashlytics**: Programmatically remove crash reports for specific users (e.g., for privacy compliance).

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
firebase-admin-sdk = "0.1.0" # Replace with actual version
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
futures = "0.3" # Required for Firestore Listen API streams
```

## Usage

### Initialization

The entry point for the SDK is the `FirebaseApp` struct. You will need a service account JSON file from the Firebase Console.

```rust
use firebase_admin_sdk::{FirebaseApp, yup_oauth2};

#[tokio::main]
async fn main() {
    // Load the service account key (e.g., from a file)
    let service_account_key = yup_oauth2::read_service_account_key("service-account.json").await.unwrap();

    let app = FirebaseApp::new(service_account_key);

    // Access services
    let auth = app.auth();
    let messaging = app.messaging();
    let remote_config = app.remote_config();
    let firestore = app.firestore();
}
```

### Authentication Example

```rust
use firebase_admin_sdk::auth::models::CreateUserRequest;

async fn create_user(app: &firebase_admin_sdk::FirebaseApp) {
    let auth = app.auth();

    let request = CreateUserRequest {
        uid: Some("test-uid".to_string()),
        email: Some("test@example.com".to_string()),
        ..Default::default()
    };

    match auth.create_user(request).await {
        Ok(user) => println!("Created user: {:?}", user.uid),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Cloud Messaging Example

```rust
use firebase_admin_sdk::messaging::models::{Message, Notification};

async fn send_message(app: &firebase_admin_sdk::FirebaseApp) {
    let messaging = app.messaging();

    let message = Message {
        token: Some("device_token".to_string()),
        notification: Some(Notification {
            title: Some("Hello".to_string()),
            body: Some("World".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    match messaging.send(&message).await {
        Ok(id) => println!("Sent message: {}", id),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Remote Config Example

```rust
async fn get_config(app: &firebase_admin_sdk::FirebaseApp) {
    let rc = app.remote_config();

    match rc.get().await {
        Ok(config) => println!("Current config: {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Firestore Example

```rust
use serde_json::json;

async fn add_document(app: &firebase_admin_sdk::FirebaseApp) {
    let firestore = app.firestore();
    let doc_ref = firestore.collection("users").doc("user1");

    let data = json!({
        "name": "John Doe",
        "age": 30
    });

    match doc_ref.set(&data).await {
        Ok(_) => println!("Document saved"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Firestore Listen Example (Real-time Updates)

You can listen for real-time updates on a document or an entire collection using the `listen()` method.

```rust
use futures::stream::StreamExt;

async fn listen_to_document(app: &firebase_admin_sdk::FirebaseApp) {
    let firestore = app.firestore();
    let doc_ref = firestore.collection("users").doc("user1");

    let mut stream = doc_ref.listen().await.expect("Failed to create stream");

    while let Some(response_result) = stream.next().await {
        match response_result {
            Ok(response) => {
                if let Some(change) = response.document_change {
                    println!("Document changed: {:?}", change.document);
                }
            }
            Err(e) => eprintln!("Stream error: {}", e),
        }
    }
}
```

### Storage Example

```rust
async fn upload_file(app: &firebase_admin_sdk::FirebaseApp) {
    let storage = app.storage();
    let bucket = storage.bucket(None); // Uses default bucket

    let file_content = b"Hello, World!";
    let file = bucket.file("hello.txt");

    match file.save(file_content, "text/plain").await {
        Ok(_) => println!("File uploaded"),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Generate a V4 signed URL
    let options = firebase_admin_sdk::storage::GetSignedUrlOptions {
        method: firebase_admin_sdk::storage::SignedUrlMethod::GET,
        expires: std::time::SystemTime::now() + std::time::Duration::from_secs(3600), // 1 hour
        content_type: None,
    };

    match file.get_signed_url(options) {
        Ok(url) => println!("Signed URL: {}", url),
        Err(e) => eprintln!("Error generating signed URL: {}", e),
    }
}
```

### Crashlytics Example

```rust
async fn delete_reports(app: &firebase_admin_sdk::FirebaseApp) {
    let crashlytics = app.crashlytics();

    match crashlytics.delete_crash_reports("1:1234567890:android:321abc456def7890", "user_uid").await {
        Ok(_) => println!("Crash reports deleted"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Roadmap & TODO

- [ ] **Realtime Database**: Implement support for reading and writing to the Firebase Realtime Database.
- [ ] **Machine Learning**: Add support for Firebase ML Model management.
- [x] **Auth**: Add support for OIDC and SAML provider configuration.
- [ ] **Storage**: Add support for setting and updating object metadata.
- [ ] **General**: Improve error handling and add more comprehensive integration tests.

## Architecture and Patterns

This SDK is designed with specific architectural patterns to ensure usability, performance, and reliability.

### 1. Synchronous Constructor, Lazy Asynchronous Authentication

One of the key design goals was to keep the `FirebaseApp::new()` constructor synchronous. In Rust, async constructors are not idiomatic and can complicate application startup logic.

However, Firebase APIs require OAuth2 authentication, which involves asynchronous HTTP requests to fetch tokens. To bridge this gap, we use a **lazy initialization pattern**:

-   `FirebaseApp::new()` simply stores the `ServiceAccountKey` and returns immediately.
-   The HTTP client uses a custom `AuthMiddleware`.
-   This middleware contains a `tokio::sync::OnceCell` that holds the authenticator.
-   The first time any API request is made (e.g., `auth.get_user(...)`), the middleware checks the `OnceCell`. If it's empty, it asynchronously initializes the OAuth2 authenticator and fetches a token.
-   Subsequent requests reuse the initialized authenticator and cached tokens.

### 2. Middleware Stack

The SDK leverages `reqwest_middleware` to build a robust HTTP client stack:

1.  **Retry Middleware**: Automatically retries failed requests (with exponential backoff) for transient errors (e.g., network blips, 5xx server errors).
2.  **Auth Middleware**: Transparently handles OAuth2 token acquisition and injection into the `Authorization` header.

This separation of concerns keeps the business logic in the service modules (Auth, Messaging, etc.) clean and focused on the Firebase APIs themselves.

### 3. Factory Pattern

`FirebaseApp` acts as a lightweight factory and configuration holder.

-   Calling `app.auth()` or `app.messaging()` creates a new, lightweight client instance.
-   These instances share the underlying `ServiceAccountKey` (cloned cheaply) but are otherwise independent.
-   This allows you to easily create clients where needed without worrying about complex lifecycle management.

### 4. Type-Safe API

Where possible, the SDK uses strong typing to prevent runtime errors:

-   **FCM**: The `Message` struct uses generic builders or strictly typed fields to ensure valid payloads.
-   **Remote Config**: The `ETag` is handled automatically to ensure safe concurrent updates (`If-Match` headers).
-   **Firestore**: The `DocumentReference` and `CollectionReference` types mirror the path structure of your database.

## License

[MIT](LICENSE)
