// Authentication Response DTOs
//
// These DTOs represent the output data for authentication operations.
// They implement Serialize for JSON responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Registration Response
// =============================================================================

/// Response returned after successful user registration.
///
/// Contains the created user's public information (no password hash).
///
/// Requirements: 7.1
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterResponse {
    /// The unique identifier of the created user
    pub user_id: Uuid,
    /// The user's username (auto-generated for ecommerce, provided for POS)
    pub username: String,
    /// The user's email address
    pub email: String,
    /// The user's first name
    pub first_name: String,
    /// The user's last name
    pub last_name: String,
    /// Timestamp when the user was created (ISO 8601 format)
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl RegisterResponse {
    /// Creates a new RegisterResponse.
    pub fn new(
        user_id: Uuid,
        username: String,
        email: String,
        first_name: String,
        last_name: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            username,
            email,
            first_name,
            last_name,
            created_at,
        }
    }
}

// =============================================================================
// Login Response
// =============================================================================

/// Response returned after successful login.
///
/// Contains the JWT tokens for authenticated requests.
///
/// Requirements: 7.2
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginResponse {
    /// JWT access token for API requests
    pub access_token: String,
    /// Refresh token for obtaining new access tokens
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Access token lifetime in seconds
    pub expires_in: i64,
}

impl LoginResponse {
    /// Creates a new LoginResponse with Bearer token type.
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

// =============================================================================
// Error Response
// =============================================================================

/// Response returned when an error occurs.
///
/// Provides a consistent error format for all API errors.
///
/// Requirements: 7.3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorResponse {
    /// Machine-readable error code (e.g., "INVALID_CREDENTIALS")
    pub error_code: String,
    /// Human-readable error message
    pub message: String,
}

impl ErrorResponse {
    /// Creates a new ErrorResponse.
    pub fn new(error_code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_code: error_code.into(),
            message: message.into(),
        }
    }

    // -------------------------------------------------------------------------
    // Common error constructors
    // -------------------------------------------------------------------------

    /// Creates an invalid credentials error.
    pub fn invalid_credentials() -> Self {
        Self::new("INVALID_CREDENTIALS", "Invalid credentials")
    }

    /// Creates an account disabled error.
    pub fn account_disabled() -> Self {
        Self::new("ACCOUNT_DISABLED", "Account is disabled")
    }

    /// Creates a token expired error.
    pub fn token_expired() -> Self {
        Self::new("TOKEN_EXPIRED", "Token has expired")
    }

    /// Creates an invalid token error.
    pub fn invalid_token() -> Self {
        Self::new("INVALID_TOKEN", "Invalid token")
    }

    /// Creates a validation error with custom message.
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    /// Creates a duplicate email error.
    pub fn duplicate_email() -> Self {
        Self::new("DUPLICATE_EMAIL", "Email already registered")
    }

    /// Creates a duplicate username error.
    pub fn duplicate_username() -> Self {
        Self::new("DUPLICATE_USERNAME", "Username already taken")
    }

    /// Creates a store not found error.
    pub fn store_not_found() -> Self {
        Self::new("STORE_NOT_FOUND", "Store not found or inactive")
    }

    /// Creates an internal error (without exposing details).
    pub fn internal_error() -> Self {
        Self::new("INTERNAL_ERROR", "Internal error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_register_response_serialize() {
        let response = RegisterResponse::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            "john_doe".to_string(),
            "john@example.com".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            Utc.with_ymd_and_hms(2025, 1, 13, 10, 30, 0).unwrap(),
        );

        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["user_id"], "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(json["username"], "john_doe");
        assert_eq!(json["email"], "john@example.com");
        assert_eq!(json["first_name"], "John");
        assert_eq!(json["last_name"], "Doe");
        // created_at is serialized as Unix timestamp
        assert!(json["created_at"].is_number());
    }

    #[test]
    fn test_register_response_round_trip() {
        let response = RegisterResponse::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            "john_doe".to_string(),
            "john@example.com".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            Utc.with_ymd_and_hms(2025, 1, 13, 10, 30, 0).unwrap(),
        );

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: RegisterResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response, deserialized);
    }

    #[test]
    fn test_login_response_serialize() {
        let response = LoginResponse::new(
            "access_token_value".to_string(),
            "refresh_token_value".to_string(),
            900,
        );

        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["access_token"], "access_token_value");
        assert_eq!(json["refresh_token"], "refresh_token_value");
        assert_eq!(json["token_type"], "Bearer");
        assert_eq!(json["expires_in"], 900);
    }

    #[test]
    fn test_login_response_round_trip() {
        let response = LoginResponse::new(
            "eyJhbGciOiJIUzI1NiIs...".to_string(),
            "eyJhbGciOiJIUzI1NiIs...".to_string(),
            900,
        );

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: LoginResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response, deserialized);
    }

    #[test]
    fn test_error_response_serialize() {
        let response = ErrorResponse::new("INVALID_CREDENTIALS", "Invalid credentials");

        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["error_code"], "INVALID_CREDENTIALS");
        assert_eq!(json["message"], "Invalid credentials");
    }

    #[test]
    fn test_error_response_constructors() {
        assert_eq!(
            ErrorResponse::invalid_credentials().error_code,
            "INVALID_CREDENTIALS"
        );
        assert_eq!(
            ErrorResponse::account_disabled().error_code,
            "ACCOUNT_DISABLED"
        );
        assert_eq!(ErrorResponse::token_expired().error_code, "TOKEN_EXPIRED");
        assert_eq!(ErrorResponse::invalid_token().error_code, "INVALID_TOKEN");
        assert_eq!(
            ErrorResponse::validation_error("test").error_code,
            "VALIDATION_ERROR"
        );
        assert_eq!(
            ErrorResponse::duplicate_email().error_code,
            "DUPLICATE_EMAIL"
        );
        assert_eq!(
            ErrorResponse::duplicate_username().error_code,
            "DUPLICATE_USERNAME"
        );
        assert_eq!(
            ErrorResponse::store_not_found().error_code,
            "STORE_NOT_FOUND"
        );
        assert_eq!(
            ErrorResponse::internal_error().error_code,
            "INTERNAL_ERROR"
        );
    }

    #[test]
    fn test_error_response_round_trip() {
        let response = ErrorResponse::validation_error("Password too short");

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ErrorResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response, deserialized);
    }
}
