//! TaxAppliesTo enum - scope of tax rate application

use crate::FiscalError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Defines the scope of a tax rate application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxAppliesTo {
    /// Tax applies to all products
    All,
    /// Tax applies only to specific categories
    Categories,
}

impl TaxAppliesTo {
    /// Returns all available tax application scopes
    pub fn all() -> &'static [TaxAppliesTo] {
        &[TaxAppliesTo::All, TaxAppliesTo::Categories]
    }
}

impl FromStr for TaxAppliesTo {
    type Err = FiscalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(TaxAppliesTo::All),
            "categories" => Ok(TaxAppliesTo::Categories),
            _ => Err(FiscalError::InvalidTaxAppliesTo),
        }
    }
}

impl fmt::Display for TaxAppliesTo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaxAppliesTo::All => write!(f, "all"),
            TaxAppliesTo::Categories => write!(f, "categories"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(TaxAppliesTo::from_str("all").unwrap(), TaxAppliesTo::All);
        assert_eq!(
            TaxAppliesTo::from_str("categories").unwrap(),
            TaxAppliesTo::Categories
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(TaxAppliesTo::All.to_string(), "all");
        assert_eq!(TaxAppliesTo::Categories.to_string(), "categories");
    }

    #[test]
    fn test_invalid() {
        assert!(TaxAppliesTo::from_str("invalid").is_err());
    }
}
