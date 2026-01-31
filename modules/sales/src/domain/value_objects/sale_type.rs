//! SaleType enum - type of sale (POS vs Online)

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Type of sale transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaleType {
    /// Point of Sale transaction (in-store)
    Pos,
    /// Online/E-commerce transaction
    Online,
}

impl SaleType {
    /// Returns all available sale types
    pub fn all() -> &'static [SaleType] {
        &[SaleType::Pos, SaleType::Online]
    }

    /// Returns true if this is a POS sale
    pub fn is_pos(&self) -> bool {
        matches!(self, SaleType::Pos)
    }

    /// Returns true if this is an online sale
    pub fn is_online(&self) -> bool {
        matches!(self, SaleType::Online)
    }
}

impl FromStr for SaleType {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pos" => Ok(SaleType::Pos),
            "online" | "ecommerce" | "e-commerce" => Ok(SaleType::Online),
            _ => Err(SalesError::InvalidSaleStatus),
        }
    }
}

impl fmt::Display for SaleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaleType::Pos => write!(f, "pos"),
            SaleType::Online => write!(f, "online"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(SaleType::from_str("pos").unwrap(), SaleType::Pos);
        assert_eq!(SaleType::from_str("online").unwrap(), SaleType::Online);
        assert_eq!(SaleType::from_str("ecommerce").unwrap(), SaleType::Online);
    }

    #[test]
    fn test_display() {
        assert_eq!(SaleType::Pos.to_string(), "pos");
        assert_eq!(SaleType::Online.to_string(), "online");
    }

    #[test]
    fn test_predicates() {
        assert!(SaleType::Pos.is_pos());
        assert!(!SaleType::Pos.is_online());
        assert!(SaleType::Online.is_online());
        assert!(!SaleType::Online.is_pos());
    }
}
