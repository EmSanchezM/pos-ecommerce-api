//! TaxType enum - classification of tax rates for Honduras ISV

use crate::FiscalError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Classification of tax rates for Honduras ISV (Impuesto Sobre Ventas)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxType {
    /// Standard ISV rate of 15%
    Isv15,
    /// Special ISV rate of 18% (alcohol, tobacco, etc.)
    Isv18,
    /// Tax exempt
    Exempt,
}

impl TaxType {
    /// Returns all available tax types
    pub fn all() -> &'static [TaxType] {
        &[TaxType::Isv15, TaxType::Isv18, TaxType::Exempt]
    }

    /// Returns the tax rate percentage for this type
    pub fn rate(&self) -> rust_decimal::Decimal {
        use rust_decimal::Decimal;
        match self {
            TaxType::Isv15 => Decimal::new(15, 0),
            TaxType::Isv18 => Decimal::new(18, 0),
            TaxType::Exempt => Decimal::ZERO,
        }
    }

    /// Returns true if this type is tax exempt
    pub fn is_exempt(&self) -> bool {
        matches!(self, TaxType::Exempt)
    }
}

impl FromStr for TaxType {
    type Err = FiscalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "isv15" | "isv_15" | "15" => Ok(TaxType::Isv15),
            "isv18" | "isv_18" | "18" => Ok(TaxType::Isv18),
            "exempt" | "exento" => Ok(TaxType::Exempt),
            _ => Err(FiscalError::InvalidTaxType),
        }
    }
}

impl fmt::Display for TaxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaxType::Isv15 => write!(f, "isv15"),
            TaxType::Isv18 => write!(f, "isv18"),
            TaxType::Exempt => write!(f, "exempt"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_from_str() {
        assert_eq!(TaxType::from_str("isv15").unwrap(), TaxType::Isv15);
        assert_eq!(TaxType::from_str("isv_15").unwrap(), TaxType::Isv15);
        assert_eq!(TaxType::from_str("15").unwrap(), TaxType::Isv15);
        assert_eq!(TaxType::from_str("isv18").unwrap(), TaxType::Isv18);
        assert_eq!(TaxType::from_str("exempt").unwrap(), TaxType::Exempt);
        assert_eq!(TaxType::from_str("exento").unwrap(), TaxType::Exempt);
    }

    #[test]
    fn test_display() {
        assert_eq!(TaxType::Isv15.to_string(), "isv15");
        assert_eq!(TaxType::Isv18.to_string(), "isv18");
        assert_eq!(TaxType::Exempt.to_string(), "exempt");
    }

    #[test]
    fn test_rate() {
        assert_eq!(TaxType::Isv15.rate(), Decimal::new(15, 0));
        assert_eq!(TaxType::Isv18.rate(), Decimal::new(18, 0));
        assert_eq!(TaxType::Exempt.rate(), Decimal::ZERO);
    }

    #[test]
    fn test_is_exempt() {
        assert!(!TaxType::Isv15.is_exempt());
        assert!(!TaxType::Isv18.is_exempt());
        assert!(TaxType::Exempt.is_exempt());
    }
}
