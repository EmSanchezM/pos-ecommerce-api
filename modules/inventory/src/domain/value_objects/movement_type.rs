// MovementType enum - types of inventory movements (Kardex)

use crate::InventoryError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Type of inventory movement for Kardex entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementType {
    /// Stock received/added
    In,
    /// Stock sold/removed
    Out,
    /// Manual adjustment
    Adjustment,
    /// Transfer out to another store
    TransferOut,
    /// Transfer in from another store
    TransferIn,
    /// Reserved for order/cart
    Reservation,
    /// Released from reservation
    Release,
}

impl MovementType {
    /// Returns all available movement types
    pub fn all() -> &'static [MovementType] {
        &[
            MovementType::In,
            MovementType::Out,
            MovementType::Adjustment,
            MovementType::TransferOut,
            MovementType::TransferIn,
            MovementType::Reservation,
            MovementType::Release,
        ]
    }

    /// Returns true if this movement type increases stock
    pub fn is_increase(&self) -> bool {
        matches!(self, MovementType::In | MovementType::TransferIn | MovementType::Release)
    }

    /// Returns true if this movement type decreases stock
    pub fn is_decrease(&self) -> bool {
        matches!(self, MovementType::Out | MovementType::TransferOut | MovementType::Reservation)
    }
}

impl FromStr for MovementType {
    type Err = InventoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "in" | "receive" | "received" => Ok(MovementType::In),
            "out" | "sale" | "sold" => Ok(MovementType::Out),
            "adjustment" | "adjust" => Ok(MovementType::Adjustment),
            "transfer_out" | "transferout" => Ok(MovementType::TransferOut),
            "transfer_in" | "transferin" => Ok(MovementType::TransferIn),
            "reservation" | "reserve" | "reserved" => Ok(MovementType::Reservation),
            "release" | "released" => Ok(MovementType::Release),
            _ => Err(InventoryError::InvalidMovementType),
        }
    }
}

impl fmt::Display for MovementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MovementType::In => write!(f, "in"),
            MovementType::Out => write!(f, "out"),
            MovementType::Adjustment => write!(f, "adjustment"),
            MovementType::TransferOut => write!(f, "transfer_out"),
            MovementType::TransferIn => write!(f, "transfer_in"),
            MovementType::Reservation => write!(f, "reservation"),
            MovementType::Release => write!(f, "release"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(MovementType::from_str("in").unwrap(), MovementType::In);
        assert_eq!(MovementType::from_str("out").unwrap(), MovementType::Out);
        assert_eq!(MovementType::from_str("adjustment").unwrap(), MovementType::Adjustment);
        assert_eq!(MovementType::from_str("transfer_out").unwrap(), MovementType::TransferOut);
        assert_eq!(MovementType::from_str("transfer_in").unwrap(), MovementType::TransferIn);
        assert_eq!(MovementType::from_str("reservation").unwrap(), MovementType::Reservation);
        assert_eq!(MovementType::from_str("release").unwrap(), MovementType::Release);
    }

    #[test]
    fn test_from_str_aliases() {
        assert_eq!(MovementType::from_str("receive").unwrap(), MovementType::In);
        assert_eq!(MovementType::from_str("sale").unwrap(), MovementType::Out);
        assert_eq!(MovementType::from_str("reserve").unwrap(), MovementType::Reservation);
    }

    #[test]
    fn test_invalid() {
        let result = MovementType::from_str("invalid");
        assert!(matches!(result, Err(InventoryError::InvalidMovementType)));
    }

    #[test]
    fn test_display() {
        assert_eq!(MovementType::In.to_string(), "in");
        assert_eq!(MovementType::TransferOut.to_string(), "transfer_out");
    }

    #[test]
    fn test_is_increase() {
        assert!(MovementType::In.is_increase());
        assert!(MovementType::TransferIn.is_increase());
        assert!(MovementType::Release.is_increase());
        assert!(!MovementType::Out.is_increase());
    }

    #[test]
    fn test_is_decrease() {
        assert!(MovementType::Out.is_decrease());
        assert!(MovementType::TransferOut.is_decrease());
        assert!(MovementType::Reservation.is_decrease());
        assert!(!MovementType::In.is_decrease());
    }
}
