// AdjustmentType enum - type of stock adjustment (increase/decrease)

use crate::InventoryError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Type of stock adjustment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentType {
    /// Increase stock quantity
    Increase,
    /// Decrease stock quantity
    Decrease,
}

impl AdjustmentType {
    /// Returns all available adjustment types
    pub fn all() -> &'static [AdjustmentType] {
        &[AdjustmentType::Increase, AdjustmentType::Decrease]
    }

    /// Returns the sign multiplier for quantity calculations
    /// Increase = +1, Decrease = -1
    pub fn sign(&self) -> i32 {
        match self {
            AdjustmentType::Increase => 1,
            AdjustmentType::Decrease => -1,
        }
    }
}

impl FromStr for AdjustmentType {
    type Err = InventoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "increase" | "add" | "in" | "+" => Ok(AdjustmentType::Increase),
            "decrease" | "remove" | "out" | "-" => Ok(AdjustmentType::Decrease),
            _ => Err(InventoryError::InvalidAdjustmentType),
        }
    }
}

impl fmt::Display for AdjustmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdjustmentType::Increase => write!(f, "increase"),
            AdjustmentType::Decrease => write!(f, "decrease"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(AdjustmentType::from_str("increase").unwrap(), AdjustmentType::Increase);
        assert_eq!(AdjustmentType::from_str("decrease").unwrap(), AdjustmentType::Decrease);
    }

    #[test]
    fn test_from_str_aliases() {
        assert_eq!(AdjustmentType::from_str("add").unwrap(), AdjustmentType::Increase);
        assert_eq!(AdjustmentType::from_str("+").unwrap(), AdjustmentType::Increase);
        assert_eq!(AdjustmentType::from_str("remove").unwrap(), AdjustmentType::Decrease);
        assert_eq!(AdjustmentType::from_str("-").unwrap(), AdjustmentType::Decrease);
    }

    #[test]
    fn test_invalid() {
        let result = AdjustmentType::from_str("invalid");
        assert!(matches!(result, Err(InventoryError::InvalidAdjustmentType)));
    }

    #[test]
    fn test_display() {
        assert_eq!(AdjustmentType::Increase.to_string(), "increase");
        assert_eq!(AdjustmentType::Decrease.to_string(), "decrease");
    }

    #[test]
    fn test_sign() {
        assert_eq!(AdjustmentType::Increase.sign(), 1);
        assert_eq!(AdjustmentType::Decrease.sign(), -1);
    }
}
