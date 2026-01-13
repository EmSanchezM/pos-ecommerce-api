// Authentication error types for the identity module
//
// These errors cover all authentication-related failures including
// credential validation, token operations, and account status checks.

use thiserror::Error;

/// Authentication-specific errors
///
/// This enum covers all error cases that can occur during authentication
/// operations including login, registration, and token management.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// Invalid credentials - used for both wrong password and non-existent user
    /// to prevent user enumeration attacks
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// User account has been disabled/deactivated
    #[error("Account is disabled")]
    AccountDisabled,

    /// JWT token has expired
    #[error("Token expired")]
    TokenExpired,

    /// JWT token is malformed or has invalid signature
    #[error("Invalid token")]
    InvalidToken,

    /// Password does not meet minimum length requirement (8 characters)
    #[error("Password too short: minimum 8 characters required")]
    PasswordTooShort,

    /// Email format is invalid (not RFC 5322 compliant)
    #[error("Invalid email format")]
    InvalidEmailFormat,

    /// Username format is invalid (must be 3-50 chars, alphanumeric + underscore, start with letter)
    #[error("Invalid username format")]
    InvalidUsernameFormat,

    /// Email address is already registered in the system
    #[error("Email already registered: {0}")]
    DuplicateEmail(String),

    /// Username is already taken by another user
    #[error("Username already taken: {0}")]
    DuplicateUsername(String),

    /// Store does not exist or is inactive (for POS user registration)
    #[error("Store not found or inactive")]
    StoreNotFound,

    /// First name or last name validation failed
    #[error("Invalid name: {0}")]
    InvalidName(String),

    /// Internal error - wraps unexpected failures
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_credentials_message() {
        let err = AuthError::InvalidCredentials;
        assert_eq!(err.to_string(), "Invalid credentials");
    }

    #[test]
    fn test_account_disabled_message() {
        let err = AuthError::AccountDisabled;
        assert_eq!(err.to_string(), "Account is disabled");
    }

    #[test]
    fn test_token_expired_message() {
        let err = AuthError::TokenExpired;
        assert_eq!(err.to_string(), "Token expired");
    }

    #[test]
    fn test_invalid_token_message() {
        let err = AuthError::InvalidToken;
        assert_eq!(err.to_string(), "Invalid token");
    }

    #[test]
    fn test_password_too_short_message() {
        let err = AuthError::PasswordTooShort;
        assert_eq!(
            err.to_string(),
            "Password too short: minimum 8 characters required"
        );
    }

    #[test]
    fn test_duplicate_email_message() {
        let err = AuthError::DuplicateEmail("test@example.com".to_string());
        assert_eq!(err.to_string(), "Email already registered: test@example.com");
    }

    #[test]
    fn test_duplicate_username_message() {
        let err = AuthError::DuplicateUsername("john_doe".to_string());
        assert_eq!(err.to_string(), "Username already taken: john_doe");
    }

    #[test]
    fn test_store_not_found_message() {
        let err = AuthError::StoreNotFound;
        assert_eq!(err.to_string(), "Store not found or inactive");
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(AuthError::InvalidCredentials, AuthError::InvalidCredentials);
        assert_ne!(AuthError::InvalidCredentials, AuthError::AccountDisabled);
    }
}
