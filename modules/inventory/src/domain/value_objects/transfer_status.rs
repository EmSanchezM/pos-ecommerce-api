// TransferStatus enum - workflow status for stock transfers

use crate::InventoryError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Workflow status for stock transfers between stores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    /// Initial state, can be edited
    Draft,
    /// Submitted and waiting to be shipped
    Pending,
    /// Shipped, in transit to destination
    InTransit,
    /// Received and completed
    Completed,
    /// Cancelled
    Cancelled,
}

impl TransferStatus {
    /// Returns all available transfer statuses
    pub fn all() -> &'static [TransferStatus] {
        &[
            TransferStatus::Draft,
            TransferStatus::Pending,
            TransferStatus::InTransit,
            TransferStatus::Completed,
            TransferStatus::Cancelled,
        ]
    }

    /// Returns true if the transfer can be edited
    pub fn is_editable(&self) -> bool {
        matches!(self, TransferStatus::Draft)
    }

    /// Returns true if the transfer is in a final state
    pub fn is_final(&self) -> bool {
        matches!(self, TransferStatus::Completed | TransferStatus::Cancelled)
    }

    /// Returns true if the transfer can be submitted
    pub fn can_submit(&self) -> bool {
        matches!(self, TransferStatus::Draft)
    }

    /// Returns true if the transfer can be shipped
    pub fn can_ship(&self) -> bool {
        matches!(self, TransferStatus::Pending)
    }

    /// Returns true if the transfer can be received
    pub fn can_receive(&self) -> bool {
        matches!(self, TransferStatus::InTransit)
    }

    /// Returns true if the transfer can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(self, TransferStatus::Draft | TransferStatus::Pending)
    }
}

impl FromStr for TransferStatus {
    type Err = InventoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "draft" => Ok(TransferStatus::Draft),
            "pending" | "submitted" => Ok(TransferStatus::Pending),
            "in_transit" | "intransit" | "shipped" => Ok(TransferStatus::InTransit),
            "completed" | "received" => Ok(TransferStatus::Completed),
            "cancelled" | "canceled" => Ok(TransferStatus::Cancelled),
            _ => Err(InventoryError::InvalidTransferStatus),
        }
    }
}

impl fmt::Display for TransferStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferStatus::Draft => write!(f, "draft"),
            TransferStatus::Pending => write!(f, "pending"),
            TransferStatus::InTransit => write!(f, "in_transit"),
            TransferStatus::Completed => write!(f, "completed"),
            TransferStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(TransferStatus::from_str("draft").unwrap(), TransferStatus::Draft);
        assert_eq!(TransferStatus::from_str("pending").unwrap(), TransferStatus::Pending);
        assert_eq!(TransferStatus::from_str("in_transit").unwrap(), TransferStatus::InTransit);
        assert_eq!(TransferStatus::from_str("completed").unwrap(), TransferStatus::Completed);
        assert_eq!(TransferStatus::from_str("cancelled").unwrap(), TransferStatus::Cancelled);
    }

    #[test]
    fn test_from_str_aliases() {
        assert_eq!(TransferStatus::from_str("submitted").unwrap(), TransferStatus::Pending);
        assert_eq!(TransferStatus::from_str("shipped").unwrap(), TransferStatus::InTransit);
        assert_eq!(TransferStatus::from_str("intransit").unwrap(), TransferStatus::InTransit);
        assert_eq!(TransferStatus::from_str("received").unwrap(), TransferStatus::Completed);
        assert_eq!(TransferStatus::from_str("canceled").unwrap(), TransferStatus::Cancelled);
    }

    #[test]
    fn test_invalid() {
        let result = TransferStatus::from_str("invalid");
        assert!(matches!(result, Err(InventoryError::InvalidTransferStatus)));
    }

    #[test]
    fn test_display() {
        assert_eq!(TransferStatus::Draft.to_string(), "draft");
        assert_eq!(TransferStatus::InTransit.to_string(), "in_transit");
    }

    #[test]
    fn test_workflow_states() {
        // Draft state
        assert!(TransferStatus::Draft.is_editable());
        assert!(TransferStatus::Draft.can_submit());
        assert!(TransferStatus::Draft.can_cancel());
        assert!(!TransferStatus::Draft.is_final());

        // Pending state
        assert!(!TransferStatus::Pending.is_editable());
        assert!(TransferStatus::Pending.can_ship());
        assert!(TransferStatus::Pending.can_cancel());

        // In transit state
        assert!(TransferStatus::InTransit.can_receive());
        assert!(!TransferStatus::InTransit.can_cancel());

        // Final states
        assert!(TransferStatus::Completed.is_final());
        assert!(TransferStatus::Cancelled.is_final());
    }
}
