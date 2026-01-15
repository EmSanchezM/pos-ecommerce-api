// ReservationStatus enum - status of inventory reservations

use crate::InventoryError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Status of an inventory reservation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReservationStatus {
    /// Reservation is active and holding stock
    Pending,
    /// Reservation was confirmed (order placed)
    Confirmed,
    /// Reservation was cancelled
    Cancelled,
    /// Reservation expired without confirmation
    Expired,
}

impl ReservationStatus {
    /// Returns all available reservation statuses
    pub fn all() -> &'static [ReservationStatus] {
        &[
            ReservationStatus::Pending,
            ReservationStatus::Confirmed,
            ReservationStatus::Cancelled,
            ReservationStatus::Expired,
        ]
    }

    /// Returns true if the reservation is still holding stock
    pub fn is_active(&self) -> bool {
        matches!(self, ReservationStatus::Pending)
    }

    /// Returns true if the reservation has been finalized (no longer active)
    pub fn is_finalized(&self) -> bool {
        matches!(
            self,
            ReservationStatus::Confirmed | ReservationStatus::Cancelled | ReservationStatus::Expired
        )
    }
}

impl FromStr for ReservationStatus {
    type Err = InventoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" | "active" => Ok(ReservationStatus::Pending),
            "confirmed" | "completed" => Ok(ReservationStatus::Confirmed),
            "cancelled" | "canceled" => Ok(ReservationStatus::Cancelled),
            "expired" => Ok(ReservationStatus::Expired),
            _ => Err(InventoryError::InvalidReservationStatusValue),
        }
    }
}

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Confirmed => write!(f, "confirmed"),
            ReservationStatus::Cancelled => write!(f, "cancelled"),
            ReservationStatus::Expired => write!(f, "expired"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(ReservationStatus::from_str("pending").unwrap(), ReservationStatus::Pending);
        assert_eq!(ReservationStatus::from_str("confirmed").unwrap(), ReservationStatus::Confirmed);
        assert_eq!(ReservationStatus::from_str("cancelled").unwrap(), ReservationStatus::Cancelled);
        assert_eq!(ReservationStatus::from_str("expired").unwrap(), ReservationStatus::Expired);
    }

    #[test]
    fn test_from_str_aliases() {
        assert_eq!(ReservationStatus::from_str("active").unwrap(), ReservationStatus::Pending);
        assert_eq!(ReservationStatus::from_str("completed").unwrap(), ReservationStatus::Confirmed);
        assert_eq!(ReservationStatus::from_str("canceled").unwrap(), ReservationStatus::Cancelled);
    }

    #[test]
    fn test_invalid() {
        let result = ReservationStatus::from_str("invalid");
        assert!(matches!(result, Err(InventoryError::InvalidReservationStatusValue)));
    }

    #[test]
    fn test_display() {
        assert_eq!(ReservationStatus::Pending.to_string(), "pending");
        assert_eq!(ReservationStatus::Confirmed.to_string(), "confirmed");
    }

    #[test]
    fn test_is_active() {
        assert!(ReservationStatus::Pending.is_active());
        assert!(!ReservationStatus::Confirmed.is_active());
        assert!(!ReservationStatus::Cancelled.is_active());
        assert!(!ReservationStatus::Expired.is_active());
    }

    #[test]
    fn test_is_finalized() {
        assert!(!ReservationStatus::Pending.is_finalized());
        assert!(ReservationStatus::Confirmed.is_finalized());
        assert!(ReservationStatus::Cancelled.is_finalized());
        assert!(ReservationStatus::Expired.is_finalized());
    }
}
