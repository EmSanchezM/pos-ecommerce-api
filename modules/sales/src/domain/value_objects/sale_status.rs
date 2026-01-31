//! SaleStatus enum - workflow status for POS sales

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Workflow status for POS sales
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaleStatus {
    /// Initial state, sale in progress
    Draft,
    /// Sale completed and invoiced
    Completed,
    /// Sale voided before completion
    Voided,
    /// Sale has been fully returned
    Returned,
}

impl SaleStatus {
    /// Returns all available sale statuses
    pub fn all() -> &'static [SaleStatus] {
        &[
            SaleStatus::Draft,
            SaleStatus::Completed,
            SaleStatus::Voided,
            SaleStatus::Returned,
        ]
    }

    /// Returns true if the sale can be edited
    pub fn is_editable(&self) -> bool {
        matches!(self, SaleStatus::Draft)
    }

    /// Returns true if the sale is in a final state
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            SaleStatus::Completed | SaleStatus::Voided | SaleStatus::Returned
        )
    }

    /// Returns true if the sale can be completed
    pub fn can_complete(&self) -> bool {
        matches!(self, SaleStatus::Draft)
    }

    /// Returns true if the sale can be voided
    pub fn can_void(&self) -> bool {
        matches!(self, SaleStatus::Draft)
    }

    /// Returns true if a return can be created for this sale
    pub fn can_return(&self) -> bool {
        matches!(self, SaleStatus::Completed)
    }

    /// Validates transition from current status to new status
    pub fn can_transition_to(&self, new_status: SaleStatus) -> bool {
        match (self, new_status) {
            // From Draft
            (SaleStatus::Draft, SaleStatus::Completed) => true,
            (SaleStatus::Draft, SaleStatus::Voided) => true,
            // From Completed
            (SaleStatus::Completed, SaleStatus::Returned) => true,
            // All other transitions are invalid
            _ => false,
        }
    }
}

impl FromStr for SaleStatus {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(SaleStatus::Draft),
            "completed" => Ok(SaleStatus::Completed),
            "voided" | "void" => Ok(SaleStatus::Voided),
            "returned" => Ok(SaleStatus::Returned),
            _ => Err(SalesError::InvalidSaleStatus),
        }
    }
}

impl fmt::Display for SaleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaleStatus::Draft => write!(f, "draft"),
            SaleStatus::Completed => write!(f, "completed"),
            SaleStatus::Voided => write!(f, "voided"),
            SaleStatus::Returned => write!(f, "returned"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(SaleStatus::from_str("draft").unwrap(), SaleStatus::Draft);
        assert_eq!(
            SaleStatus::from_str("completed").unwrap(),
            SaleStatus::Completed
        );
        assert_eq!(SaleStatus::from_str("voided").unwrap(), SaleStatus::Voided);
        assert_eq!(SaleStatus::from_str("void").unwrap(), SaleStatus::Voided);
        assert_eq!(
            SaleStatus::from_str("returned").unwrap(),
            SaleStatus::Returned
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(SaleStatus::Draft.to_string(), "draft");
        assert_eq!(SaleStatus::Completed.to_string(), "completed");
    }

    #[test]
    fn test_workflow_states() {
        // Draft state
        assert!(SaleStatus::Draft.is_editable());
        assert!(SaleStatus::Draft.can_complete());
        assert!(SaleStatus::Draft.can_void());
        assert!(!SaleStatus::Draft.is_final());

        // Completed state
        assert!(!SaleStatus::Completed.is_editable());
        assert!(SaleStatus::Completed.can_return());
        assert!(SaleStatus::Completed.is_final());

        // Final states
        assert!(SaleStatus::Voided.is_final());
        assert!(SaleStatus::Returned.is_final());
    }

    #[test]
    fn test_valid_transitions() {
        // From Draft
        assert!(SaleStatus::Draft.can_transition_to(SaleStatus::Completed));
        assert!(SaleStatus::Draft.can_transition_to(SaleStatus::Voided));
        assert!(!SaleStatus::Draft.can_transition_to(SaleStatus::Returned));

        // From Completed
        assert!(SaleStatus::Completed.can_transition_to(SaleStatus::Returned));
        assert!(!SaleStatus::Completed.can_transition_to(SaleStatus::Voided));
    }
}
