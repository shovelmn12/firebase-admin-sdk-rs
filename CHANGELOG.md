# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-18

### Added
- **Authentication**: 
  - Support for OIDC and SAML provider configuration management.
  - Improved email action link generation (password reset, email verification, sign-in).
  - Multi-tenancy support via `TenantAwareness`.
- **Firestore**:
  - Implementation of `list_collections` for root and document levels.
  - Support for `WriteBatch` atomic operations.
  - Simplified Transaction API (removed lifetime requirements for easier async usage).
- **Storage**:
  - Implementation of V4 Signed URL generation.
  - Support for getting and setting/updating object metadata.
  - Comprehensive `Bucket` and `File` API for uploads, downloads, and deletions.
- **Crashlytics**: 
  - Implementation of the Firebase Crashlytics API for managing crash reports.
- **Core**:
  - Centralized `FirebaseErrorResponse` parsing for detailed API error messages.
  - `AuthMiddleware` refactor for efficient state sharing across services.

### Fixed
- FCM request serialization to use `camelCase` as required by the v1 API.
- Better error handling and reporting across all Firebase services.

### Testing
- Added comprehensive integration tests for Firestore Transactions, FCM Messaging, and Storage Signed URLs using `httpmock`.

## [0.1.0] - 2026-01-18

### Added
- Initial implementation of the Firebase Admin SDK for Rust.
- Basic support for Authentication (User Management, Token Verification).
- Basic support for Cloud Messaging (FCM).
- Basic support for Remote Config.
- Basic support for Firestore (Document/Collection references, Real-time Listen).
- Architecture based on Synchronous Constructor and Lazy Asynchronous Authentication.
