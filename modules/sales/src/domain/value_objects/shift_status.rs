//! ShiftStatus enum - status of a cashier shift

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Status of a cashier shift
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShiftStatus {
    /// Shift is currently open
    Open,
    /// Shift has been closed
    Closed,
}

impl ShiftStatus {
    /// Returns all available shift statuses
    pub fn all() -> &'static [ShiftStatus] {
        &[ShiftStatus::Open, ShiftStatus::Closed]
    }

    /// Returns true if the shift is open
    pub fn is_open(&self) -> bool {
        matches!(self, ShiftStatus::Open)
    }

    /// Returns true if the shift is closed
    pub fn is_closed(&self) -> bool {
        matches!(self, ShiftStatus::Closed)
    }

    /// Returns true if the shift can be closed
    pub fn can_close(&self) -> bool {
        matches!(self, ShiftStatus::Open)
    }

    /// Validates transition from current status to new status
    pub fn can_transition_to(&self, new_status: ShiftStatus) -> bool {
        match (self, new_status) {
            (ShiftStatus::Open, ShiftStatus::Closed) => true,
            _ => false,
        }
    }
}

impl FromStr for ShiftStatus {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" | "active" => Ok(ShiftStatus::Open),
            "closed" | "inactive" => Ok(ShiftStatus::Closed),
            _ => Err(SalesError::InvalidSaleStatus),
        }
    }
}

impl fmt::Display for ShiftStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShiftStatus::Open => write!(f, "open"),
            ShiftStatus::Closed => write!(f, "closed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(ShiftStatus::from_str("open").unwrap(), ShiftStatus::Open);
        assert_eq!(ShiftStatus::from_str("active").unwrap(), ShiftStatus::Open);
        assert_eq!(ShiftStatus::from_str("closed").unwrap(), ShiftStatus::Closed);
    }

    #[test]
    fn test_display() {
        assert_eq!(ShiftStatus::Open.to_string(), "open");
        assert_eq!(ShiftStatus::Closed.to_string(), "closed");
    }

    #[test]
    fn test_predicates() {
        assert!(ShiftStatus::Open.is_open());
        assert!(!ShiftStatus::Open.is_closed());
        assert!(ShiftStatus::Open.can_close());

        assert!(ShiftStatus::Closed.is_closed());
        assert!(!ShiftStatus::Closed.is_open());
        assert!(!ShiftStatus::Closed.can_close());
    }

    #[test]
    fn test_valid_transitions() {
        assert!(ShiftStatus::Open.can_transition_to(ShiftStatus::Closed));
        assert!(!ShiftStatus::Closed.can_transition_to(ShiftStatus::Open));
    }
}
