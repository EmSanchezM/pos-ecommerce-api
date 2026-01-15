// AdjustmentStatus enum - workflow status for stock adjustments

use crate::InventoryError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Workflow status for stock adjustments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentStatus {
    /// Initial state, can be edited
    Draft,
    /// Submitted for approval
    PendingApproval,
    /// Approved by authorized user
    Approved,
    /// Rejected by authorized user
    Rejected,
    /// Applied to inventory
    Applied,
}

impl AdjustmentStatus {
    /// Returns all available adjustment statuses
    pub fn all() -> &'static [AdjustmentStatus] {
        &[
            AdjustmentStatus::Draft,
            AdjustmentStatus::PendingApproval,
            AdjustmentStatus::Approved,
            AdjustmentStatus::Rejected,
            AdjustmentStatus::Applied,
        ]
    }

    /// Returns true if the adjustment can be edited
    pub fn is_editable(&self) -> bool {
        matches!(self, AdjustmentStatus::Draft)
    }

    /// Returns true if the adjustment is in a final state
    pub fn is_final(&self) -> bool {
        matches!(self, AdjustmentStatus::Rejected | AdjustmentStatus::Applied)
    }

    /// Returns true if the adjustment can be submitted for approval
    pub fn can_submit(&self) -> bool {
        matches!(self, AdjustmentStatus::Draft)
    }

    /// Returns true if the adjustment can be approved or rejected
    pub fn can_review(&self) -> bool {
        matches!(self, AdjustmentStatus::PendingApproval)
    }

    /// Returns true if the adjustment can be applied
    pub fn can_apply(&self) -> bool {
        matches!(self, AdjustmentStatus::Approved)
    }
}

impl FromStr for AdjustmentStatus {
    type Err = InventoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "draft" => Ok(AdjustmentStatus::Draft),
            "pending_approval" | "pendingapproval" | "pending" => Ok(AdjustmentStatus::PendingApproval),
            "approved" => Ok(AdjustmentStatus::Approved),
            "rejected" => Ok(AdjustmentStatus::Rejected),
            "applied" => Ok(AdjustmentStatus::Applied),
            _ => Err(InventoryError::InvalidAdjustmentStatus),
        }
    }
}

impl fmt::Display for AdjustmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdjustmentStatus::Draft => write!(f, "draft"),
            AdjustmentStatus::PendingApproval => write!(f, "pending_approval"),
            AdjustmentStatus::Approved => write!(f, "approved"),
            AdjustmentStatus::Rejected => write!(f, "rejected"),
            AdjustmentStatus::Applied => write!(f, "applied"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(AdjustmentStatus::from_str("draft").unwrap(), AdjustmentStatus::Draft);
        assert_eq!(AdjustmentStatus::from_str("pending_approval").unwrap(), AdjustmentStatus::PendingApproval);
        assert_eq!(AdjustmentStatus::from_str("approved").unwrap(), AdjustmentStatus::Approved);
        assert_eq!(AdjustmentStatus::from_str("rejected").unwrap(), AdjustmentStatus::Rejected);
        assert_eq!(AdjustmentStatus::from_str("applied").unwrap(), AdjustmentStatus::Applied);
    }

    #[test]
    fn test_from_str_aliases() {
        assert_eq!(AdjustmentStatus::from_str("pending").unwrap(), AdjustmentStatus::PendingApproval);
        assert_eq!(AdjustmentStatus::from_str("pendingapproval").unwrap(), AdjustmentStatus::PendingApproval);
    }

    #[test]
    fn test_invalid() {
        let result = AdjustmentStatus::from_str("invalid");
        assert!(matches!(result, Err(InventoryError::InvalidAdjustmentStatus)));
    }

    #[test]
    fn test_display() {
        assert_eq!(AdjustmentStatus::Draft.to_string(), "draft");
        assert_eq!(AdjustmentStatus::PendingApproval.to_string(), "pending_approval");
    }

    #[test]
    fn test_workflow_states() {
        // Draft state
        assert!(AdjustmentStatus::Draft.is_editable());
        assert!(AdjustmentStatus::Draft.can_submit());
        assert!(!AdjustmentStatus::Draft.is_final());

        // Pending approval state
        assert!(!AdjustmentStatus::PendingApproval.is_editable());
        assert!(AdjustmentStatus::PendingApproval.can_review());

        // Approved state
        assert!(AdjustmentStatus::Approved.can_apply());
        assert!(!AdjustmentStatus::Approved.is_final());

        // Final states
        assert!(AdjustmentStatus::Rejected.is_final());
        assert!(AdjustmentStatus::Applied.is_final());
    }
}
