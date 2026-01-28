// PurchaseOrderStatus enum - workflow status for purchase orders

use crate::PurchasingError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Workflow status for purchase orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseOrderStatus {
    /// Initial state, can be edited
    Draft,
    /// Submitted for approval
    Submitted,
    /// Approved, pending receipt of goods
    Approved,
    /// Partially received goods
    PartiallyReceived,
    /// Completely received all goods
    Received,
    /// Closed (finalized)
    Closed,
    /// Cancelled
    Cancelled,
}

impl PurchaseOrderStatus {
    /// Returns all available purchase order statuses
    pub fn all() -> &'static [PurchaseOrderStatus] {
        &[
            PurchaseOrderStatus::Draft,
            PurchaseOrderStatus::Submitted,
            PurchaseOrderStatus::Approved,
            PurchaseOrderStatus::PartiallyReceived,
            PurchaseOrderStatus::Received,
            PurchaseOrderStatus::Closed,
            PurchaseOrderStatus::Cancelled,
        ]
    }

    /// Returns true if the order can be edited
    pub fn is_editable(&self) -> bool {
        matches!(self, PurchaseOrderStatus::Draft)
    }

    /// Returns true if the order is in a final state
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            PurchaseOrderStatus::Closed | PurchaseOrderStatus::Cancelled
        )
    }

    /// Returns true if the order can be submitted for approval
    pub fn can_submit(&self) -> bool {
        matches!(self, PurchaseOrderStatus::Draft)
    }

    /// Returns true if the order can be approved or rejected
    pub fn can_review(&self) -> bool {
        matches!(self, PurchaseOrderStatus::Submitted)
    }

    /// Returns true if goods can be received for this order
    pub fn can_receive(&self) -> bool {
        matches!(
            self,
            PurchaseOrderStatus::Approved | PurchaseOrderStatus::PartiallyReceived
        )
    }

    /// Returns true if the order can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(
            self,
            PurchaseOrderStatus::Draft | PurchaseOrderStatus::Submitted
        )
    }

    /// Returns true if the order can be closed
    pub fn can_close(&self) -> bool {
        matches!(self, PurchaseOrderStatus::Received)
    }

    /// Validates transition from current status to new status
    pub fn can_transition_to(&self, new_status: PurchaseOrderStatus) -> bool {
        match (self, new_status) {
            // From Draft
            (PurchaseOrderStatus::Draft, PurchaseOrderStatus::Submitted) => true,
            (PurchaseOrderStatus::Draft, PurchaseOrderStatus::Cancelled) => true,
            // From Submitted
            (PurchaseOrderStatus::Submitted, PurchaseOrderStatus::Approved) => true,
            (PurchaseOrderStatus::Submitted, PurchaseOrderStatus::Draft) => true, // Rejection
            (PurchaseOrderStatus::Submitted, PurchaseOrderStatus::Cancelled) => true,
            // From Approved
            (PurchaseOrderStatus::Approved, PurchaseOrderStatus::PartiallyReceived) => true,
            (PurchaseOrderStatus::Approved, PurchaseOrderStatus::Received) => true,
            // From PartiallyReceived
            (PurchaseOrderStatus::PartiallyReceived, PurchaseOrderStatus::Received) => true,
            // From Received
            (PurchaseOrderStatus::Received, PurchaseOrderStatus::Closed) => true,
            // All other transitions are invalid
            _ => false,
        }
    }
}

impl FromStr for PurchaseOrderStatus {
    type Err = PurchasingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "draft" => Ok(PurchaseOrderStatus::Draft),
            "submitted" => Ok(PurchaseOrderStatus::Submitted),
            "approved" => Ok(PurchaseOrderStatus::Approved),
            "partially_received" | "partiallyreceived" | "partial" => {
                Ok(PurchaseOrderStatus::PartiallyReceived)
            }
            "received" => Ok(PurchaseOrderStatus::Received),
            "closed" => Ok(PurchaseOrderStatus::Closed),
            "cancelled" | "canceled" => Ok(PurchaseOrderStatus::Cancelled),
            _ => Err(PurchasingError::InvalidPurchaseOrderStatus),
        }
    }
}

impl fmt::Display for PurchaseOrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PurchaseOrderStatus::Draft => write!(f, "draft"),
            PurchaseOrderStatus::Submitted => write!(f, "submitted"),
            PurchaseOrderStatus::Approved => write!(f, "approved"),
            PurchaseOrderStatus::PartiallyReceived => write!(f, "partially_received"),
            PurchaseOrderStatus::Received => write!(f, "received"),
            PurchaseOrderStatus::Closed => write!(f, "closed"),
            PurchaseOrderStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            PurchaseOrderStatus::from_str("draft").unwrap(),
            PurchaseOrderStatus::Draft
        );
        assert_eq!(
            PurchaseOrderStatus::from_str("submitted").unwrap(),
            PurchaseOrderStatus::Submitted
        );
        assert_eq!(
            PurchaseOrderStatus::from_str("approved").unwrap(),
            PurchaseOrderStatus::Approved
        );
        assert_eq!(
            PurchaseOrderStatus::from_str("partially_received").unwrap(),
            PurchaseOrderStatus::PartiallyReceived
        );
        assert_eq!(
            PurchaseOrderStatus::from_str("received").unwrap(),
            PurchaseOrderStatus::Received
        );
        assert_eq!(
            PurchaseOrderStatus::from_str("closed").unwrap(),
            PurchaseOrderStatus::Closed
        );
        assert_eq!(
            PurchaseOrderStatus::from_str("cancelled").unwrap(),
            PurchaseOrderStatus::Cancelled
        );
    }

    #[test]
    fn test_from_str_aliases() {
        assert_eq!(
            PurchaseOrderStatus::from_str("partial").unwrap(),
            PurchaseOrderStatus::PartiallyReceived
        );
        assert_eq!(
            PurchaseOrderStatus::from_str("canceled").unwrap(),
            PurchaseOrderStatus::Cancelled
        );
    }

    #[test]
    fn test_invalid() {
        let result = PurchaseOrderStatus::from_str("invalid");
        assert!(matches!(
            result,
            Err(PurchasingError::InvalidPurchaseOrderStatus)
        ));
    }

    #[test]
    fn test_display() {
        assert_eq!(PurchaseOrderStatus::Draft.to_string(), "draft");
        assert_eq!(
            PurchaseOrderStatus::PartiallyReceived.to_string(),
            "partially_received"
        );
    }

    #[test]
    fn test_workflow_states() {
        // Draft state
        assert!(PurchaseOrderStatus::Draft.is_editable());
        assert!(PurchaseOrderStatus::Draft.can_submit());
        assert!(PurchaseOrderStatus::Draft.can_cancel());
        assert!(!PurchaseOrderStatus::Draft.is_final());

        // Submitted state
        assert!(!PurchaseOrderStatus::Submitted.is_editable());
        assert!(PurchaseOrderStatus::Submitted.can_review());
        assert!(PurchaseOrderStatus::Submitted.can_cancel());

        // Approved state
        assert!(PurchaseOrderStatus::Approved.can_receive());
        assert!(!PurchaseOrderStatus::Approved.can_cancel());

        // PartiallyReceived state
        assert!(PurchaseOrderStatus::PartiallyReceived.can_receive());

        // Received state
        assert!(PurchaseOrderStatus::Received.can_close());

        // Final states
        assert!(PurchaseOrderStatus::Closed.is_final());
        assert!(PurchaseOrderStatus::Cancelled.is_final());
    }

    #[test]
    fn test_valid_transitions() {
        // From Draft
        assert!(PurchaseOrderStatus::Draft.can_transition_to(PurchaseOrderStatus::Submitted));
        assert!(PurchaseOrderStatus::Draft.can_transition_to(PurchaseOrderStatus::Cancelled));
        assert!(!PurchaseOrderStatus::Draft.can_transition_to(PurchaseOrderStatus::Approved));

        // From Submitted
        assert!(PurchaseOrderStatus::Submitted.can_transition_to(PurchaseOrderStatus::Approved));
        assert!(PurchaseOrderStatus::Submitted.can_transition_to(PurchaseOrderStatus::Draft));
        assert!(PurchaseOrderStatus::Submitted.can_transition_to(PurchaseOrderStatus::Cancelled));

        // From Approved
        assert!(
            PurchaseOrderStatus::Approved.can_transition_to(PurchaseOrderStatus::PartiallyReceived)
        );
        assert!(PurchaseOrderStatus::Approved.can_transition_to(PurchaseOrderStatus::Received));
        assert!(!PurchaseOrderStatus::Approved.can_transition_to(PurchaseOrderStatus::Cancelled));

        // From Received
        assert!(PurchaseOrderStatus::Received.can_transition_to(PurchaseOrderStatus::Closed));
    }
}
