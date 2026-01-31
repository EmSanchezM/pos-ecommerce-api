//! CreditNoteStatus enum - workflow status for credit notes/returns

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Workflow status for credit notes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditNoteStatus {
    /// Initial state, can be edited
    Draft,
    /// Submitted for approval
    Pending,
    /// Approved by supervisor
    Approved,
    /// Refund processed and applied
    Applied,
    /// Cancelled
    Cancelled,
}

impl CreditNoteStatus {
    /// Returns all available credit note statuses
    pub fn all() -> &'static [CreditNoteStatus] {
        &[
            CreditNoteStatus::Draft,
            CreditNoteStatus::Pending,
            CreditNoteStatus::Approved,
            CreditNoteStatus::Applied,
            CreditNoteStatus::Cancelled,
        ]
    }

    /// Returns true if the credit note can be edited
    pub fn is_editable(&self) -> bool {
        matches!(self, CreditNoteStatus::Draft)
    }

    /// Returns true if the credit note is in a final state
    pub fn is_final(&self) -> bool {
        matches!(self, CreditNoteStatus::Applied | CreditNoteStatus::Cancelled)
    }

    /// Returns true if the credit note can be submitted
    pub fn can_submit(&self) -> bool {
        matches!(self, CreditNoteStatus::Draft)
    }

    /// Returns true if the credit note can be approved
    pub fn can_approve(&self) -> bool {
        matches!(self, CreditNoteStatus::Pending)
    }

    /// Returns true if the refund can be applied
    pub fn can_apply(&self) -> bool {
        matches!(self, CreditNoteStatus::Approved)
    }

    /// Returns true if the credit note can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(self, CreditNoteStatus::Draft | CreditNoteStatus::Pending)
    }

    /// Validates transition from current status to new status
    pub fn can_transition_to(&self, new_status: CreditNoteStatus) -> bool {
        match (self, new_status) {
            // From Draft
            (CreditNoteStatus::Draft, CreditNoteStatus::Pending) => true,
            (CreditNoteStatus::Draft, CreditNoteStatus::Cancelled) => true,
            // From Pending
            (CreditNoteStatus::Pending, CreditNoteStatus::Approved) => true,
            (CreditNoteStatus::Pending, CreditNoteStatus::Cancelled) => true,
            // From Approved
            (CreditNoteStatus::Approved, CreditNoteStatus::Applied) => true,
            // All other transitions are invalid
            _ => false,
        }
    }
}

impl FromStr for CreditNoteStatus {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(CreditNoteStatus::Draft),
            "pending" | "submitted" => Ok(CreditNoteStatus::Pending),
            "approved" => Ok(CreditNoteStatus::Approved),
            "applied" | "processed" => Ok(CreditNoteStatus::Applied),
            "cancelled" | "canceled" => Ok(CreditNoteStatus::Cancelled),
            _ => Err(SalesError::InvalidCreditNoteStatus),
        }
    }
}

impl fmt::Display for CreditNoteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CreditNoteStatus::Draft => write!(f, "draft"),
            CreditNoteStatus::Pending => write!(f, "pending"),
            CreditNoteStatus::Approved => write!(f, "approved"),
            CreditNoteStatus::Applied => write!(f, "applied"),
            CreditNoteStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            CreditNoteStatus::from_str("draft").unwrap(),
            CreditNoteStatus::Draft
        );
        assert_eq!(
            CreditNoteStatus::from_str("pending").unwrap(),
            CreditNoteStatus::Pending
        );
        assert_eq!(
            CreditNoteStatus::from_str("submitted").unwrap(),
            CreditNoteStatus::Pending
        );
        assert_eq!(
            CreditNoteStatus::from_str("cancelled").unwrap(),
            CreditNoteStatus::Cancelled
        );
        assert_eq!(
            CreditNoteStatus::from_str("canceled").unwrap(),
            CreditNoteStatus::Cancelled
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(CreditNoteStatus::Draft.to_string(), "draft");
        assert_eq!(CreditNoteStatus::Applied.to_string(), "applied");
    }

    #[test]
    fn test_workflow_states() {
        // Draft state
        assert!(CreditNoteStatus::Draft.is_editable());
        assert!(CreditNoteStatus::Draft.can_submit());
        assert!(CreditNoteStatus::Draft.can_cancel());
        assert!(!CreditNoteStatus::Draft.is_final());

        // Pending state
        assert!(!CreditNoteStatus::Pending.is_editable());
        assert!(CreditNoteStatus::Pending.can_approve());
        assert!(CreditNoteStatus::Pending.can_cancel());

        // Approved state
        assert!(CreditNoteStatus::Approved.can_apply());
        assert!(!CreditNoteStatus::Approved.can_cancel());

        // Final states
        assert!(CreditNoteStatus::Applied.is_final());
        assert!(CreditNoteStatus::Cancelled.is_final());
    }

    #[test]
    fn test_valid_transitions() {
        // From Draft
        assert!(CreditNoteStatus::Draft.can_transition_to(CreditNoteStatus::Pending));
        assert!(CreditNoteStatus::Draft.can_transition_to(CreditNoteStatus::Cancelled));
        assert!(!CreditNoteStatus::Draft.can_transition_to(CreditNoteStatus::Approved));

        // From Pending
        assert!(CreditNoteStatus::Pending.can_transition_to(CreditNoteStatus::Approved));
        assert!(CreditNoteStatus::Pending.can_transition_to(CreditNoteStatus::Cancelled));

        // From Approved
        assert!(CreditNoteStatus::Approved.can_transition_to(CreditNoteStatus::Applied));
        assert!(!CreditNoteStatus::Approved.can_transition_to(CreditNoteStatus::Cancelled));
    }
}
