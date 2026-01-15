// Barcode value object - optional product barcode with max 100 chars

use crate::InventoryError;
use serde::{Deserialize, Serialize};

/// Product barcode - optional, max 100 characters
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Barcode(String);

impl Barcode {
    /// Maximum allowed length for a barcode
    pub const MAX_LENGTH: usize = 100;

    /// Creates a new Barcode, validating the length
    pub fn new(value: &str) -> Result<Self, InventoryError> {
        if value.len() > Self::MAX_LENGTH {
            return Err(InventoryError::InvalidBarcode);
        }
        Ok(Self(value.to_string()))
    }

    /// Reconstitutes a Barcode from database (no validation needed)
    pub fn from_string(value: String) -> Self {
        Self(value)
    }

    /// Returns the barcode as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Barcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barcode_valid() {
        let barcode = Barcode::new("1234567890123").unwrap();
        assert_eq!(barcode.as_str(), "1234567890123");
    }

    #[test]
    fn test_barcode_max_length() {
        let value = "a".repeat(100);
        let barcode = Barcode::new(&value).unwrap();
        assert_eq!(barcode.as_str().len(), 100);
    }

    #[test]
    fn test_barcode_too_long() {
        let value = "a".repeat(101);
        let result = Barcode::new(&value);
        assert!(matches!(result, Err(InventoryError::InvalidBarcode)));
    }

    #[test]
    fn test_barcode_empty() {
        let barcode = Barcode::new("").unwrap();
        assert_eq!(barcode.as_str(), "");
    }
}
