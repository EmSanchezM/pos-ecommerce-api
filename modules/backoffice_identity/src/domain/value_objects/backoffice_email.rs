use crate::error::BackofficeIdentityError;
use serde::{Deserialize, Serialize};

/// Validated email address for backoffice users.
/// Same validation rules as identity::Email.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackofficeEmail(String);

impl BackofficeEmail {
    pub fn new(email: &str) -> Result<Self, BackofficeIdentityError> {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(BackofficeIdentityError::InvalidEmailFormat);
        }

        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() {
            return Err(BackofficeIdentityError::InvalidEmailFormat);
        }

        if domain.is_empty() || !domain.contains('.') {
            return Err(BackofficeIdentityError::InvalidEmailFormat);
        }

        let domain_parts: Vec<&str> = domain.split('.').collect();
        let tld = domain_parts.last().unwrap();
        if tld.len() < 2 {
            return Err(BackofficeIdentityError::InvalidEmailFormat);
        }

        if domain_parts.iter().any(|p| p.is_empty()) {
            return Err(BackofficeIdentityError::InvalidEmailFormat);
        }

        Ok(Self(email.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BackofficeEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for BackofficeEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let email = BackofficeEmail::new("admin@example.com").unwrap();
        assert_eq!(email.as_str(), "admin@example.com");
    }

    #[test]
    fn test_email_normalized_to_lowercase() {
        let email = BackofficeEmail::new("Admin@Example.COM").unwrap();
        assert_eq!(email.as_str(), "admin@example.com");
    }

    #[test]
    fn test_invalid_no_at() {
        assert!(BackofficeEmail::new("adminexample.com").is_err());
    }

    #[test]
    fn test_invalid_multiple_at() {
        assert!(BackofficeEmail::new("admin@@example.com").is_err());
    }

    #[test]
    fn test_invalid_empty_local() {
        assert!(BackofficeEmail::new("@example.com").is_err());
    }

    #[test]
    fn test_invalid_no_dot_in_domain() {
        assert!(BackofficeEmail::new("admin@example").is_err());
    }

    #[test]
    fn test_invalid_short_tld() {
        assert!(BackofficeEmail::new("admin@example.c").is_err());
    }

    #[test]
    fn test_invalid_empty() {
        assert!(BackofficeEmail::new("").is_err());
    }
}
