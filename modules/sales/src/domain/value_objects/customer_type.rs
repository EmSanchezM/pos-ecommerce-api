//! CustomerType enum - type of customer

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Type of customer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomerType {
    /// Individual person
    Individual,
    /// Business/Company
    Business,
}

impl CustomerType {
    /// Returns all available customer types
    pub fn all() -> &'static [CustomerType] {
        &[CustomerType::Individual, CustomerType::Business]
    }

    /// Returns true if this is an individual customer
    pub fn is_individual(&self) -> bool {
        matches!(self, CustomerType::Individual)
    }

    /// Returns true if this is a business customer
    pub fn is_business(&self) -> bool {
        matches!(self, CustomerType::Business)
    }
}

impl Default for CustomerType {
    fn default() -> Self {
        CustomerType::Individual
    }
}

impl FromStr for CustomerType {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "individual" | "person" | "personal" => Ok(CustomerType::Individual),
            "business" | "company" | "corporate" | "enterprise" => Ok(CustomerType::Business),
            _ => Err(SalesError::InvalidCustomerType),
        }
    }
}

impl fmt::Display for CustomerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomerType::Individual => write!(f, "individual"),
            CustomerType::Business => write!(f, "business"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            CustomerType::from_str("individual").unwrap(),
            CustomerType::Individual
        );
        assert_eq!(
            CustomerType::from_str("person").unwrap(),
            CustomerType::Individual
        );
        assert_eq!(
            CustomerType::from_str("business").unwrap(),
            CustomerType::Business
        );
        assert_eq!(
            CustomerType::from_str("company").unwrap(),
            CustomerType::Business
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(CustomerType::Individual.to_string(), "individual");
        assert_eq!(CustomerType::Business.to_string(), "business");
    }

    #[test]
    fn test_default() {
        assert_eq!(CustomerType::default(), CustomerType::Individual);
    }

    #[test]
    fn test_predicates() {
        assert!(CustomerType::Individual.is_individual());
        assert!(!CustomerType::Individual.is_business());
        assert!(CustomerType::Business.is_business());
        assert!(!CustomerType::Business.is_individual());
    }
}
