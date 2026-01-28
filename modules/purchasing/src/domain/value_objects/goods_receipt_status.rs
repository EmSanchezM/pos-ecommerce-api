// GoodsReceiptStatus enum - workflow status for goods receipts

use crate::PurchasingError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Workflow status for goods receipts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoodsReceiptStatus {
    /// Initial state, can be edited
    Draft,
    /// Confirmed and applied to inventory
    Confirmed,
    /// Cancelled
    Cancelled,
}

impl GoodsReceiptStatus {
    /// Returns all available goods receipt statuses
    pub fn all() -> &'static [GoodsReceiptStatus] {
        &[
            GoodsReceiptStatus::Draft,
            GoodsReceiptStatus::Confirmed,
            GoodsReceiptStatus::Cancelled,
        ]
    }

    /// Returns true if the receipt can be edited
    pub fn is_editable(&self) -> bool {
        matches!(self, GoodsReceiptStatus::Draft)
    }

    /// Returns true if the receipt is in a final state
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            GoodsReceiptStatus::Confirmed | GoodsReceiptStatus::Cancelled
        )
    }

    /// Returns true if the receipt can be confirmed
    pub fn can_confirm(&self) -> bool {
        matches!(self, GoodsReceiptStatus::Draft)
    }

    /// Returns true if the receipt can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(self, GoodsReceiptStatus::Draft)
    }

    /// Validates transition from current status to new status
    pub fn can_transition_to(&self, new_status: GoodsReceiptStatus) -> bool {
        match (self, new_status) {
            // From Draft
            (GoodsReceiptStatus::Draft, GoodsReceiptStatus::Confirmed) => true,
            (GoodsReceiptStatus::Draft, GoodsReceiptStatus::Cancelled) => true,
            // All other transitions are invalid
            _ => false,
        }
    }
}

impl FromStr for GoodsReceiptStatus {
    type Err = PurchasingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "draft" => Ok(GoodsReceiptStatus::Draft),
            "confirmed" => Ok(GoodsReceiptStatus::Confirmed),
            "cancelled" | "canceled" => Ok(GoodsReceiptStatus::Cancelled),
            _ => Err(PurchasingError::InvalidGoodsReceiptStatus),
        }
    }
}

impl fmt::Display for GoodsReceiptStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GoodsReceiptStatus::Draft => write!(f, "draft"),
            GoodsReceiptStatus::Confirmed => write!(f, "confirmed"),
            GoodsReceiptStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            GoodsReceiptStatus::from_str("draft").unwrap(),
            GoodsReceiptStatus::Draft
        );
        assert_eq!(
            GoodsReceiptStatus::from_str("confirmed").unwrap(),
            GoodsReceiptStatus::Confirmed
        );
        assert_eq!(
            GoodsReceiptStatus::from_str("cancelled").unwrap(),
            GoodsReceiptStatus::Cancelled
        );
    }

    #[test]
    fn test_from_str_aliases() {
        assert_eq!(
            GoodsReceiptStatus::from_str("canceled").unwrap(),
            GoodsReceiptStatus::Cancelled
        );
    }

    #[test]
    fn test_invalid() {
        let result = GoodsReceiptStatus::from_str("invalid");
        assert!(matches!(
            result,
            Err(PurchasingError::InvalidGoodsReceiptStatus)
        ));
    }

    #[test]
    fn test_display() {
        assert_eq!(GoodsReceiptStatus::Draft.to_string(), "draft");
        assert_eq!(GoodsReceiptStatus::Confirmed.to_string(), "confirmed");
        assert_eq!(GoodsReceiptStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_workflow_states() {
        // Draft state
        assert!(GoodsReceiptStatus::Draft.is_editable());
        assert!(GoodsReceiptStatus::Draft.can_confirm());
        assert!(GoodsReceiptStatus::Draft.can_cancel());
        assert!(!GoodsReceiptStatus::Draft.is_final());

        // Final states
        assert!(GoodsReceiptStatus::Confirmed.is_final());
        assert!(!GoodsReceiptStatus::Confirmed.is_editable());
        assert!(GoodsReceiptStatus::Cancelled.is_final());
    }

    #[test]
    fn test_valid_transitions() {
        // From Draft
        assert!(GoodsReceiptStatus::Draft.can_transition_to(GoodsReceiptStatus::Confirmed));
        assert!(GoodsReceiptStatus::Draft.can_transition_to(GoodsReceiptStatus::Cancelled));

        // From final states
        assert!(!GoodsReceiptStatus::Confirmed.can_transition_to(GoodsReceiptStatus::Cancelled));
        assert!(!GoodsReceiptStatus::Cancelled.can_transition_to(GoodsReceiptStatus::Confirmed));
    }
}
