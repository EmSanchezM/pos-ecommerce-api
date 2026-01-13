// LoginIdentifier value object - represents either email or username for login
//
// This enum allows the login system to accept either an email address or
// a username as the identifier, automatically detecting the format.

use crate::domain::value_objects::{Email, Username};

/// Represents a login identifier which can be either an email or username.
///
/// The system automatically detects the format based on whether the input
/// matches email format (contains @ with valid domain) or username format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginIdentifier {
    /// Login identifier is a valid email address
    Email(Email),
    /// Login identifier is a username
    Username(Username),
}

impl LoginIdentifier {
    /// Parses a string into either Email or Username based on format.
    ///
    /// The parsing logic:
    /// 1. First attempts to parse as Email (checks for @ and valid domain)
    /// 2. If email parsing fails, attempts to parse as Username
    /// 3. If both fail, returns the raw string wrapped in a fallback
    ///
    /// # Arguments
    ///
    /// * `identifier` - The login identifier string (email or username)
    ///
    /// # Returns
    ///
    /// A `LoginIdentifier` enum variant based on the detected format.
    /// If the identifier doesn't match either format, it defaults to
    /// treating it as a username attempt (which may fail validation later).
    ///
    /// # Examples
    ///
    /// ```
    /// use identity::domain::auth::LoginIdentifier;
    ///
    /// // Email format detected
    /// let id = LoginIdentifier::parse("user@example.com");
    /// assert!(matches!(id, LoginIdentifier::Email(_)));
    ///
    /// // Username format detected
    /// let id = LoginIdentifier::parse("john_doe");
    /// assert!(matches!(id, LoginIdentifier::Username(_)));
    /// ```
    pub fn parse(identifier: &str) -> Self {
        // First, try to parse as email
        if let Ok(email) = Email::new(identifier) {
            return LoginIdentifier::Email(email);
        }

        // If not a valid email, try to parse as username
        if let Ok(username) = Username::new(identifier) {
            return LoginIdentifier::Username(username);
        }

        // If neither works, we still need to return something.
        // Create a "raw" username that will fail validation later.
        // This allows the authentication flow to continue and return
        // a proper "invalid credentials" error.
        LoginIdentifier::Username(Username::new_unchecked(identifier))
    }

    /// Returns true if this identifier is an email
    pub fn is_email(&self) -> bool {
        matches!(self, LoginIdentifier::Email(_))
    }

    /// Returns true if this identifier is a username
    pub fn is_username(&self) -> bool {
        matches!(self, LoginIdentifier::Username(_))
    }

    /// Returns the identifier as a string slice
    pub fn as_str(&self) -> &str {
        match self {
            LoginIdentifier::Email(email) => email.as_str(),
            LoginIdentifier::Username(username) => username.as_str(),
        }
    }
}

impl std::fmt::Display for LoginIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginIdentifier::Email(email) => write!(f, "{}", email),
            LoginIdentifier::Username(username) => write!(f, "{}", username),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_email() {
        let id = LoginIdentifier::parse("user@example.com");
        assert!(id.is_email());
        assert!(!id.is_username());
        assert_eq!(id.as_str(), "user@example.com");
    }

    #[test]
    fn test_parse_valid_email_with_subdomain() {
        let id = LoginIdentifier::parse("user@mail.example.com");
        assert!(id.is_email());
    }

    #[test]
    fn test_parse_valid_username() {
        let id = LoginIdentifier::parse("john_doe");
        assert!(id.is_username());
        assert!(!id.is_email());
        assert_eq!(id.as_str(), "john_doe");
    }

    #[test]
    fn test_parse_username_with_numbers() {
        let id = LoginIdentifier::parse("user123");
        assert!(id.is_username());
    }

    #[test]
    fn test_parse_short_string_as_username() {
        // "abc" is valid username (3 chars minimum)
        let id = LoginIdentifier::parse("abc");
        assert!(id.is_username());
    }

    #[test]
    fn test_parse_email_takes_precedence() {
        // If it looks like an email, it's parsed as email
        let id = LoginIdentifier::parse("a@b.co");
        assert!(id.is_email());
    }

    #[test]
    fn test_parse_invalid_email_without_domain() {
        // "user@" is not a valid email, should be treated as username attempt
        let id = LoginIdentifier::parse("user@");
        assert!(id.is_username());
    }

    #[test]
    fn test_display_email() {
        let id = LoginIdentifier::parse("user@example.com");
        assert_eq!(format!("{}", id), "user@example.com");
    }

    #[test]
    fn test_display_username() {
        let id = LoginIdentifier::parse("john_doe");
        assert_eq!(format!("{}", id), "john_doe");
    }

    #[test]
    fn test_equality() {
        let id1 = LoginIdentifier::parse("user@example.com");
        let id2 = LoginIdentifier::parse("user@example.com");
        let id3 = LoginIdentifier::parse("john_doe");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}
