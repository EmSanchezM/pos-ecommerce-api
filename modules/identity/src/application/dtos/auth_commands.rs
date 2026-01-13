// Authentication Command DTOs
//
// These DTOs represent the input data for authentication operations.
// They use primitive types to keep the application boundary clean
// and allow validation in use cases.

use serde::Deserialize;
use uuid::Uuid;

// =============================================================================
// Registration Commands
// =============================================================================

/// Command to register a new ecommerce user (customer).
///
/// This is for public self-registration. The username will be
/// auto-generated from the email prefix.
///
/// Requirements: 1.1
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterEcommerceCommand {
    /// User's email address (must be unique)
    pub email: String,
    /// Plain text password (minimum 8 characters)
    pub password: String,
    /// User's first name (1-100 characters)
    pub first_name: String,
    /// User's last name (1-100 characters)
    pub last_name: String,
}

/// Command to register a new POS user (store employee).
///
/// This is for administrative registration only. Requires specifying
/// a username and store assignment.
///
/// Requirements: 2.1
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterPosCommand {
    /// Unique username (3-50 chars, alphanumeric with underscores, starts with letter)
    pub username: String,
    /// User's email address (must be unique)
    pub email: String,
    /// Plain text password (minimum 8 characters)
    pub password: String,
    /// User's first name (1-100 characters)
    pub first_name: String,
    /// User's last name (1-100 characters)
    pub last_name: String,
    /// Store ID to assign the user to
    pub store_id: Uuid,
}

// =============================================================================
// Login Command
// =============================================================================

/// Command to authenticate a user.
///
/// The identifier can be either an email or username. The system
/// will automatically detect the format and search accordingly.
///
/// Requirements: 3.1
#[derive(Debug, Clone, Deserialize)]
pub struct LoginCommand {
    /// Login identifier - can be email or username
    pub identifier: String,
    /// Plain text password
    pub password: String,
}

// =============================================================================
// Token Refresh Command
// =============================================================================

/// Command to refresh an access token using a refresh token.
///
/// Requirements: 4.5
#[derive(Debug, Clone, Deserialize)]
pub struct RefreshCommand {
    /// The refresh token obtained from login
    pub refresh_token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_ecommerce_command_deserialize() {
        let json = r#"{
            "email": "user@example.com",
            "password": "securepass123",
            "first_name": "John",
            "last_name": "Doe"
        }"#;

        let cmd: RegisterEcommerceCommand = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.email, "user@example.com");
        assert_eq!(cmd.password, "securepass123");
        assert_eq!(cmd.first_name, "John");
        assert_eq!(cmd.last_name, "Doe");
    }

    #[test]
    fn test_register_pos_command_deserialize() {
        let json = r#"{
            "username": "store_employee1",
            "email": "employee@store.com",
            "password": "securepass123",
            "first_name": "Jane",
            "last_name": "Smith",
            "store_id": "550e8400-e29b-41d4-a716-446655440000"
        }"#;

        let cmd: RegisterPosCommand = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.username, "store_employee1");
        assert_eq!(cmd.email, "employee@store.com");
        assert_eq!(cmd.password, "securepass123");
        assert_eq!(cmd.first_name, "Jane");
        assert_eq!(cmd.last_name, "Smith");
        assert_eq!(
            cmd.store_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn test_login_command_deserialize() {
        let json = r#"{
            "identifier": "user@example.com",
            "password": "securepass123"
        }"#;

        let cmd: LoginCommand = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.identifier, "user@example.com");
        assert_eq!(cmd.password, "securepass123");
    }

    #[test]
    fn test_login_command_with_username() {
        let json = r#"{
            "identifier": "john_doe",
            "password": "securepass123"
        }"#;

        let cmd: LoginCommand = serde_json::from_str(json).unwrap();

        assert_eq!(cmd.identifier, "john_doe");
        assert_eq!(cmd.password, "securepass123");
    }

    #[test]
    fn test_refresh_command_deserialize() {
        let json = r#"{
            "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
        }"#;

        let cmd: RefreshCommand = serde_json::from_str(json).unwrap();

        assert_eq!(
            cmd.refresh_token,
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
        );
    }
}
