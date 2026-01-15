// UnitOfMeasure value object - enumeration of measurement units

use crate::InventoryError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Unit of measure for products
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UnitOfMeasure {
    /// Individual unit/piece
    #[default]
    Unit,
    /// Kilogram
    Kg,
    /// Pound
    Lb,
    /// Liter
    Liter,
    /// Ounce
    Oz,
}

impl UnitOfMeasure {
    /// Returns all available units of measure
    pub fn all() -> &'static [UnitOfMeasure] {
        &[
            UnitOfMeasure::Unit,
            UnitOfMeasure::Kg,
            UnitOfMeasure::Lb,
            UnitOfMeasure::Liter,
            UnitOfMeasure::Oz,
        ]
    }
}

impl FromStr for UnitOfMeasure {
    type Err = InventoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unit" | "units" | "pcs" | "piece" | "pieces" => Ok(UnitOfMeasure::Unit),
            "kg" | "kilogram" | "kilograms" => Ok(UnitOfMeasure::Kg),
            "lb" | "lbs" | "pound" | "pounds" => Ok(UnitOfMeasure::Lb),
            "liter" | "liters" | "l" | "litre" | "litres" => Ok(UnitOfMeasure::Liter),
            "oz" | "ounce" | "ounces" => Ok(UnitOfMeasure::Oz),
            _ => Err(InventoryError::InvalidUnitOfMeasure),
        }
    }
}

impl fmt::Display for UnitOfMeasure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnitOfMeasure::Unit => write!(f, "unit"),
            UnitOfMeasure::Kg => write!(f, "kg"),
            UnitOfMeasure::Lb => write!(f, "lb"),
            UnitOfMeasure::Liter => write!(f, "liter"),
            UnitOfMeasure::Oz => write!(f, "oz"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_from_str() {
        assert_eq!(UnitOfMeasure::from_str("unit").unwrap(), UnitOfMeasure::Unit);
        assert_eq!(UnitOfMeasure::from_str("pcs").unwrap(), UnitOfMeasure::Unit);
        assert_eq!(UnitOfMeasure::from_str("piece").unwrap(), UnitOfMeasure::Unit);
    }

    #[test]
    fn test_kg_from_str() {
        assert_eq!(UnitOfMeasure::from_str("kg").unwrap(), UnitOfMeasure::Kg);
        assert_eq!(UnitOfMeasure::from_str("kilogram").unwrap(), UnitOfMeasure::Kg);
        assert_eq!(UnitOfMeasure::from_str("KG").unwrap(), UnitOfMeasure::Kg);
    }

    #[test]
    fn test_lb_from_str() {
        assert_eq!(UnitOfMeasure::from_str("lb").unwrap(), UnitOfMeasure::Lb);
        assert_eq!(UnitOfMeasure::from_str("lbs").unwrap(), UnitOfMeasure::Lb);
        assert_eq!(UnitOfMeasure::from_str("pound").unwrap(), UnitOfMeasure::Lb);
    }

    #[test]
    fn test_liter_from_str() {
        assert_eq!(UnitOfMeasure::from_str("liter").unwrap(), UnitOfMeasure::Liter);
        assert_eq!(UnitOfMeasure::from_str("l").unwrap(), UnitOfMeasure::Liter);
        assert_eq!(UnitOfMeasure::from_str("litre").unwrap(), UnitOfMeasure::Liter);
    }

    #[test]
    fn test_oz_from_str() {
        assert_eq!(UnitOfMeasure::from_str("oz").unwrap(), UnitOfMeasure::Oz);
        assert_eq!(UnitOfMeasure::from_str("ounce").unwrap(), UnitOfMeasure::Oz);
    }

    #[test]
    fn test_invalid_unit() {
        let result = UnitOfMeasure::from_str("invalid");
        assert!(matches!(result, Err(InventoryError::InvalidUnitOfMeasure)));
    }

    #[test]
    fn test_display() {
        assert_eq!(UnitOfMeasure::Unit.to_string(), "unit");
        assert_eq!(UnitOfMeasure::Kg.to_string(), "kg");
        assert_eq!(UnitOfMeasure::Lb.to_string(), "lb");
        assert_eq!(UnitOfMeasure::Liter.to_string(), "liter");
        assert_eq!(UnitOfMeasure::Oz.to_string(), "oz");
    }

    #[test]
    fn test_default() {
        assert_eq!(UnitOfMeasure::default(), UnitOfMeasure::Unit);
    }
}
