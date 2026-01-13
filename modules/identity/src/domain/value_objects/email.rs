// Email value object - validated email address

use crate::IdentityError;
use serde::{Deserialize, Serialize};

/// Validated email address
/// 
/// Performs basic email format validation ensuring the presence of
/// exactly one @ symbol with non-empty local and domain parts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    /// Creates a new Email after validating the format.
    /// 
    /// # Validation Rules
    /// - Must contain exactly one @ symbol
    /// - Local part (before @) must be non-empty
    /// - Domain part (after @) must be non-empty and contain at least one dot
    /// - Domain must have a valid TLD (at least 2 characters after the last dot)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use identity::domain::value_objects::Email;
    /// 
    /// let email = Email::new("user@example.com").unwrap();
    /// assert_eq!(email.as_str(), "user@example.com");
    /// ```
    pub fn new(email: &str) -> Result<Self, IdentityError> {
        // Split by @ - must have exactly 2 parts
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(IdentityError::InvalidEmailFormat);
        }
        
        let local = parts[0];
        let domain = parts[1];
        
        // Local part must be non-empty
        if local.is_empty() {
            return Err(IdentityError::InvalidEmailFormat);
        }
        
        // Domain must be non-empty and contain at least one dot
        if domain.is_empty() || !domain.contains('.') {
            return Err(IdentityError::InvalidEmailFormat);
        }
        
        // Domain must have valid TLD (at least 2 chars after last dot)
        let domain_parts: Vec<&str> = domain.split('.').collect();
        let tld = domain_parts.last().unwrap();
        if tld.len() < 2 {
            return Err(IdentityError::InvalidEmailFormat);
        }
        
        // Domain parts must all be non-empty
        if domain_parts.iter().any(|p| p.is_empty()) {
            return Err(IdentityError::InvalidEmailFormat);
        }
        
        // Store email in lowercase for consistency
        Ok(Self(email.to_lowercase()))
    }

    /// Returns the email as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the local part of the email (before @)
    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap()
    }

    /// Returns the domain part of the email (after @)
    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap()
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_valid_email_with_subdomain() {
        let email = Email::new("user@mail.example.com").unwrap();
        assert_eq!(email.as_str(), "user@mail.example.com");
    }

    #[test]
    fn test_valid_email_with_plus() {
        let email = Email::new("user+tag@example.com").unwrap();
        assert_eq!(email.as_str(), "user+tag@example.com");
    }

    #[test]
    fn test_valid_email_with_dots_in_local() {
        let email = Email::new("first.last@example.com").unwrap();
        assert_eq!(email.as_str(), "first.last@example.com");
    }

    #[test]
    fn test_email_normalized_to_lowercase() {
        let email = Email::new("User@Example.COM").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_email_local_part() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(email.local_part(), "user");
    }

    #[test]
    fn test_email_domain() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(email.domain(), "example.com");
    }

    #[test]
    fn test_invalid_no_at() {
        let result = Email::new("userexample.com");
        assert!(matches!(result, Err(IdentityError::InvalidEmailFormat)));
    }

    #[test]
    fn test_invalid_multiple_at() {
        let result = Email::new("user@@example.com");
        assert!(matches!(result, Err(IdentityError::InvalidEmailFormat)));
    }

    #[test]
    fn test_invalid_empty_local() {
        let result = Email::new("@example.com");
        assert!(matches!(result, Err(IdentityError::InvalidEmailFormat)));
    }

    #[test]
    fn test_invalid_empty_domain() {
        let result = Email::new("user@");
        assert!(matches!(result, Err(IdentityError::InvalidEmailFormat)));
    }

    #[test]
    fn test_invalid_no_dot_in_domain() {
        let result = Email::new("user@example");
        assert!(matches!(result, Err(IdentityError::InvalidEmailFormat)));
    }

    #[test]
    fn test_invalid_short_tld() {
        let result = Email::new("user@example.c");
        assert!(matches!(result, Err(IdentityError::InvalidEmailFormat)));
    }

    #[test]
    fn test_invalid_empty_domain_part() {
        let result = Email::new("user@.com");
        assert!(matches!(result, Err(IdentityError::InvalidEmailFormat)));
    }

    #[test]
    fn test_invalid_empty() {
        let result = Email::new("");
        assert!(matches!(result, Err(IdentityError::InvalidEmailFormat)));
    }

    #[test]
    fn test_email_equality() {
        let e1 = Email::new("user@example.com").unwrap();
        let e2 = Email::new("user@example.com").unwrap();
        let e3 = Email::new("other@example.com").unwrap();
        
        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
    }

    #[test]
    fn test_email_equality_case_insensitive() {
        let e1 = Email::new("User@Example.com").unwrap();
        let e2 = Email::new("user@example.com").unwrap();
        
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_email_display() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(format!("{}", email), "user@example.com");
    }
}
