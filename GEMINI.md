# Project: Firebase Admin SDK for Rust

## Overview
This is a Rust implementation of the Firebase Admin SDK, enabling interaction with Firebase services (Auth, FCM, Remote Config, Firestore, Storage, Crashlytics) from a Rust backend.

## Architectural Guidelines

### 1. Synchronous Constructor, Lazy Asynchronous Authentication
- **Requirement**: The `FirebaseApp::new()` constructor **must** remain synchronous.
- **Implementation**: Do not fetch tokens in the constructor. Use a lazy initialization pattern within `AuthMiddleware`.
- **Mechanism**: The middleware should use a `tokio::sync::OnceCell` to initialize the authenticator and fetch the token only upon the first API request.

### 2. Middleware Stack
- Use `reqwest_middleware` to handle cross-cutting concerns.
- **Retry Logic**: Automatically retry failed requests for transient errors.
- **Auth Injection**: Transparently handle OAuth2 token acquisition and header injection.
- **Separation of Concerns**: Keep HTTP mechanics out of the service business logic.

### 3. Factory Pattern
- `FirebaseApp` acts as a factory and configuration holder.
- Service accessors (e.g., `app.auth()`) should return lightweight client instances.
- **State Sharing**: These instances must share the underlying `ServiceAccountKey` and token cache (via `Arc`) to avoid redundant authentication.

### 4. Type-Safe API
- Prefer strong typing over raw JSON.
- Use builder patterns or strictly typed structs for complex inputs (e.g., FCM Messages).
- Mirror the official Node.js Admin SDK API structure where idiomatic in Rust.

## Coding Standards

- **Idiomatic Rust**: Use `Result` for error handling, `Option` for nullable fields, and `serde` for serialization.
- **Formatting**: All code must be formatted using `rustfmt`.
- **Linter**: Ensure code passes `cargo clippy`.

### Testing
- **Command**: Run tests using `cargo test`.
- **Mocking**:
  - Do not rely on valid service account keys for unit tests.
  - For Authentication logic, expose internal constructors (e.g., `new_with_client`) to inject mocked `ClientWithMiddleware` or use `httpmock`.

## Git Conventions

- **Conventional Commits**: You **MUST** use [Conventional Commits](https://www.conventionalcommits.org/) for all commit messages.
- **Format**: `<type>(<scope>): <description>`
- **Types**:
  - `feat`: A new feature
  - `fix`: A bug fix
  - `docs`: Documentation only changes
  - `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc)
  - `refactor`: A code change that neither fixes a bug nor adds a feature
  - `perf`: A code change that improves performance
  - `test`: Adding missing tests or correcting existing tests
  - `chore`: Changes to the build process or auxiliary tools and libraries such as documentation generation
- **Examples**:
  - `feat(auth): add support for custom token creation`
  - `fix(firestore): correct retry logic for network timeouts`
  - `docs: add GEMINI.md context file`

## Interaction Guide

- **Build Project**: `cargo build`
- **Run Tests**: `cargo test`
- **Check formatting**: `cargo fmt --check`
