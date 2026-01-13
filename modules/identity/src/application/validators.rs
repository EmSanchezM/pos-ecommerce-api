// Validation functions for authentication and user data
//
// These validators enforce business rules for user input data
// including password policy and name validation.

use crate::domain::auth::AuthError;

/// Minimum password length required by the system
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// Maximum name length allowed for first_name and last_name
pub const MAX_NAME_LENGTH: usize = 100;

/// Validates that a password meets the minimum length requirement.
///
/// # Arguments
/// * `password` - The password string to validate
///
/// # Returns
/// * `Ok(())` if the password meets the policy
/// * `Err(AuthError::PasswordTooShort)` if the password is too short
///
/// # Requirements
/// - Minimum 8 characters (Requirements 1.4, 6.3)
///
/// # Examples
/// ```
/// use identity::application::validators::validate_password;
///
/// assert!(validate_password("securepass123").is_ok());
/// assert!(validate_password("short").is_err());
/// ```
pub fn validate_password(password: &str) -> Result<(), AuthError> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(AuthError::PasswordTooShort);
    }
    Ok(())
}

/// Validates that a name (first_name or last_name) meets the requirements.
///
/// # Arguments
/// * `name` - The name string to validate
/// * `field_name` - The field name for error messages (e.g., "first_name", "last_name")
///
/// # Returns
/// * `Ok(())` if the name is valid
/// * `Err(AuthError::InvalidName)` if the name is invalid
///
/// # Requirements
/// - 1-100 characters after trimming (Requirement 6.4)
/// - Non-empty after trimming
///
/// # Examples
/// ```
/// use identity::application::validators::validate_name;
///
/// assert!(validate_name("John", "first_name").is_ok());
/// assert!(validate_name("   ", "first_name").is_err());
/// ```
pub fn validate_name(name: &str, field_name: &str) -> Result<(), AuthError> {
    let trimmed = name.trim();
    
    if trimmed.is_empty() {
        return Err(AuthError::InvalidName(format!(
            "{} cannot be empty",
            field_name
        )));
    }
    
    if trimmed.len() > MAX_NAME_LENGTH {
        return Err(AuthError::InvalidName(format!(
            "{} cannot exceed {} characters",
            field_name, MAX_NAME_LENGTH
        )));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Password Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_password_valid_minimum_length() {
        // Exactly 8 characters should pass
        assert!(validate_password("12345678").is_ok());
    }

    #[test]
    fn test_validate_password_valid_longer() {
        // Longer passwords should pass
        assert!(validate_password("securepassword123").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        // 7 characters should fail
        assert!(matches!(
            validate_password("1234567"),
            Err(AuthError::PasswordTooShort)
        ));
    }

    #[test]
    fn test_validate_password_empty() {
        // Empty password should fail
        assert!(matches!(
            validate_password(""),
            Err(AuthError::PasswordTooShort)
        ));
    }

    #[test]
    fn test_validate_password_one_char() {
        // Single character should fail
        assert!(matches!(
            validate_password("a"),
            Err(AuthError::PasswordTooShort)
        ));
    }

    // =========================================================================
    // Name Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("John", "first_name").is_ok());
        assert!(validate_name("Doe", "last_name").is_ok());
    }

    #[test]
    fn test_validate_name_valid_with_spaces() {
        // Names with leading/trailing spaces should be valid after trim
        assert!(validate_name("  John  ", "first_name").is_ok());
    }

    #[test]
    fn test_validate_name_single_char() {
        // Single character names are valid
        assert!(validate_name("J", "first_name").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        let result = validate_name("", "first_name");
        assert!(matches!(result, Err(AuthError::InvalidName(_))));
        if let Err(AuthError::InvalidName(msg)) = result {
            assert!(msg.contains("first_name"));
            assert!(msg.contains("empty"));
        }
    }

    #[test]
    fn test_validate_name_only_whitespace() {
        let result = validate_name("   ", "last_name");
        assert!(matches!(result, Err(AuthError::InvalidName(_))));
        if let Err(AuthError::InvalidName(msg)) = result {
            assert!(msg.contains("last_name"));
            assert!(msg.contains("empty"));
        }
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(101);
        let result = validate_name(&long_name, "first_name");
        assert!(matches!(result, Err(AuthError::InvalidName(_))));
        if let Err(AuthError::InvalidName(msg)) = result {
            assert!(msg.contains("first_name"));
            assert!(msg.contains("100"));
        }
    }

    #[test]
    fn test_validate_name_exactly_max_length() {
        let max_name = "a".repeat(100);
        assert!(validate_name(&max_name, "first_name").is_ok());
    }
}
