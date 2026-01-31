//! DiscountType enum - type of discount applied

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Type of discount applied to a sale or item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscountType {
    /// Percentage discount (e.g., 10% off)
    Percentage,
    /// Fixed amount discount (e.g., $5 off)
    Fixed,
}

impl DiscountType {
    /// Returns all available discount types
    pub fn all() -> &'static [DiscountType] {
        &[DiscountType::Percentage, DiscountType::Fixed]
    }

    /// Returns true if this is a percentage discount
    pub fn is_percentage(&self) -> bool {
        matches!(self, DiscountType::Percentage)
    }

    /// Returns true if this is a fixed amount discount
    pub fn is_fixed(&self) -> bool {
        matches!(self, DiscountType::Fixed)
    }
}

impl FromStr for DiscountType {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "percentage" | "percent" | "%" => Ok(DiscountType::Percentage),
            "fixed" | "amount" | "flat" => Ok(DiscountType::Fixed),
            _ => Err(SalesError::InvalidDiscountType),
        }
    }
}

impl fmt::Display for DiscountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscountType::Percentage => write!(f, "percentage"),
            DiscountType::Fixed => write!(f, "fixed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            DiscountType::from_str("percentage").unwrap(),
            DiscountType::Percentage
        );
        assert_eq!(
            DiscountType::from_str("percent").unwrap(),
            DiscountType::Percentage
        );
        assert_eq!(DiscountType::from_str("%").unwrap(), DiscountType::Percentage);
        assert_eq!(DiscountType::from_str("fixed").unwrap(), DiscountType::Fixed);
        assert_eq!(DiscountType::from_str("amount").unwrap(), DiscountType::Fixed);
    }

    #[test]
    fn test_display() {
        assert_eq!(DiscountType::Percentage.to_string(), "percentage");
        assert_eq!(DiscountType::Fixed.to_string(), "fixed");
    }

    #[test]
    fn test_predicates() {
        assert!(DiscountType::Percentage.is_percentage());
        assert!(!DiscountType::Percentage.is_fixed());
        assert!(DiscountType::Fixed.is_fixed());
        assert!(!DiscountType::Fixed.is_percentage());
    }
}
