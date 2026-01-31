//! ReturnType enum - type of return (full or partial)

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Type of return
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReturnType {
    /// Full return of all items
    Full,
    /// Partial return of some items
    Partial,
}

impl ReturnType {
    /// Returns all available return types
    pub fn all() -> &'static [ReturnType] {
        &[ReturnType::Full, ReturnType::Partial]
    }

    /// Returns true if this is a full return
    pub fn is_full(&self) -> bool {
        matches!(self, ReturnType::Full)
    }

    /// Returns true if this is a partial return
    pub fn is_partial(&self) -> bool {
        matches!(self, ReturnType::Partial)
    }
}

impl FromStr for ReturnType {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "full" | "complete" | "total" => Ok(ReturnType::Full),
            "partial" | "some" => Ok(ReturnType::Partial),
            _ => Err(SalesError::InvalidReturnReason),
        }
    }
}

impl fmt::Display for ReturnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReturnType::Full => write!(f, "full"),
            ReturnType::Partial => write!(f, "partial"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(ReturnType::from_str("full").unwrap(), ReturnType::Full);
        assert_eq!(ReturnType::from_str("complete").unwrap(), ReturnType::Full);
        assert_eq!(ReturnType::from_str("partial").unwrap(), ReturnType::Partial);
    }

    #[test]
    fn test_display() {
        assert_eq!(ReturnType::Full.to_string(), "full");
        assert_eq!(ReturnType::Partial.to_string(), "partial");
    }

    #[test]
    fn test_predicates() {
        assert!(ReturnType::Full.is_full());
        assert!(!ReturnType::Full.is_partial());
        assert!(ReturnType::Partial.is_partial());
        assert!(!ReturnType::Partial.is_full());
    }
}
