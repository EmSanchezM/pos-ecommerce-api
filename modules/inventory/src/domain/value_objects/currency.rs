// Currency value object - ISO 4217 currency code (3 uppercase letters)

use crate::InventoryError;
use serde::{Deserialize, Serialize};

/// ISO 4217 currency code - exactly 3 uppercase ASCII letters
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Currency(String);

impl Currency {
    /// Creates a new Currency, validating the format (3 uppercase ASCII letters)
    pub fn new(code: &str) -> Result<Self, InventoryError> {
        let code = code.to_uppercase();
        if code.len() != 3 || !code.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(InventoryError::InvalidCurrency);
        }
        Ok(Self(code))
    }

    /// Reconstitutes a Currency from database (no validation needed)
    pub fn from_string(value: String) -> Self {
        Self(value)
    }

    /// Creates HNL (Honduran Lempira) currency
    pub fn hnl() -> Self {
        Self("HNL".to_string())
    }

    /// Creates USD (US Dollar) currency
    pub fn usd() -> Self {
        Self("USD".to_string())
    }

    /// Creates EUR (Euro) currency
    pub fn eur() -> Self {
        Self("EUR".to_string())
    }

    /// Returns the currency code as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self::hnl()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_valid() {
        let currency = Currency::new("USD").unwrap();
        assert_eq!(currency.as_str(), "USD");
    }

    #[test]
    fn test_currency_lowercase_converted() {
        let currency = Currency::new("usd").unwrap();
        assert_eq!(currency.as_str(), "USD");
    }

    #[test]
    fn test_currency_mixed_case() {
        let currency = Currency::new("Eur").unwrap();
        assert_eq!(currency.as_str(), "EUR");
    }

    #[test]
    fn test_currency_too_short() {
        let result = Currency::new("US");
        assert!(matches!(result, Err(InventoryError::InvalidCurrency)));
    }

    #[test]
    fn test_currency_too_long() {
        let result = Currency::new("USDD");
        assert!(matches!(result, Err(InventoryError::InvalidCurrency)));
    }

    #[test]
    fn test_currency_with_numbers() {
        let result = Currency::new("US1");
        assert!(matches!(result, Err(InventoryError::InvalidCurrency)));
    }

    #[test]
    fn test_currency_with_special_chars() {
        let result = Currency::new("US$");
        assert!(matches!(result, Err(InventoryError::InvalidCurrency)));
    }

    #[test]
    fn test_currency_hnl() {
        let currency = Currency::hnl();
        assert_eq!(currency.as_str(), "HNL");
    }

    #[test]
    fn test_currency_default() {
        let currency = Currency::default();
        assert_eq!(currency.as_str(), "HNL");
    }
}
