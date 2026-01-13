// Username value object - validated username string

use crate::IdentityError;
use serde::{Deserialize, Serialize};

/// Validated username for user identification
/// 
/// Usernames must be 3-50 characters long and contain only
/// alphanumeric characters and underscores.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Username(String);

impl Username {
    /// Creates a new Username after validating the format.
    /// 
    /// # Validation Rules
    /// - Must be 3-50 characters long
    /// - Must contain only alphanumeric characters (a-z, A-Z, 0-9) and underscores
    /// - Must start with a letter
    /// 
    /// # Examples
    /// 
    /// ```
    /// use identity::domain::value_objects::Username;
    /// 
    /// let username = Username::new("john_doe").unwrap();
    /// assert_eq!(username.as_str(), "john_doe");
    /// ```
    pub fn new(username: &str) -> Result<Self, IdentityError> {
        // Check length
        if username.len() < 3 || username.len() > 50 {
            return Err(IdentityError::InvalidUsernameFormat);
        }
        
        // Check that it starts with a letter
        let first_char = username.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() {
            return Err(IdentityError::InvalidUsernameFormat);
        }
        
        // Check all characters are alphanumeric or underscore
        if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(IdentityError::InvalidUsernameFormat);
        }
        
        Ok(Self(username.to_string()))
    }

    /// Returns the username as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_username() {
        let username = Username::new("john_doe").unwrap();
        assert_eq!(username.as_str(), "john_doe");
    }

    #[test]
    fn test_valid_username_alphanumeric() {
        let username = Username::new("user123").unwrap();
        assert_eq!(username.as_str(), "user123");
    }

    #[test]
    fn test_valid_username_min_length() {
        let username = Username::new("abc").unwrap();
        assert_eq!(username.as_str(), "abc");
    }

    #[test]
    fn test_valid_username_max_length() {
        let long_name = "a".repeat(50);
        let username = Username::new(&long_name).unwrap();
        assert_eq!(username.as_str(), long_name);
    }

    #[test]
    fn test_invalid_too_short() {
        let result = Username::new("ab");
        assert!(matches!(result, Err(IdentityError::InvalidUsernameFormat)));
    }

    #[test]
    fn test_invalid_too_long() {
        let long_name = "a".repeat(51);
        let result = Username::new(&long_name);
        assert!(matches!(result, Err(IdentityError::InvalidUsernameFormat)));
    }

    #[test]
    fn test_invalid_starts_with_number() {
        let result = Username::new("123user");
        assert!(matches!(result, Err(IdentityError::InvalidUsernameFormat)));
    }

    #[test]
    fn test_invalid_starts_with_underscore() {
        let result = Username::new("_user");
        assert!(matches!(result, Err(IdentityError::InvalidUsernameFormat)));
    }

    #[test]
    fn test_invalid_special_characters() {
        let result = Username::new("user@name");
        assert!(matches!(result, Err(IdentityError::InvalidUsernameFormat)));
    }

    #[test]
    fn test_invalid_spaces() {
        let result = Username::new("user name");
        assert!(matches!(result, Err(IdentityError::InvalidUsernameFormat)));
    }

    #[test]
    fn test_invalid_empty() {
        let result = Username::new("");
        assert!(matches!(result, Err(IdentityError::InvalidUsernameFormat)));
    }

    #[test]
    fn test_username_equality() {
        let u1 = Username::new("john").unwrap();
        let u2 = Username::new("john").unwrap();
        let u3 = Username::new("jane").unwrap();
        
        assert_eq!(u1, u2);
        assert_ne!(u1, u3);
    }

    #[test]
    fn test_username_display() {
        let username = Username::new("john_doe").unwrap();
        assert_eq!(format!("{}", username), "john_doe");
    }
}
