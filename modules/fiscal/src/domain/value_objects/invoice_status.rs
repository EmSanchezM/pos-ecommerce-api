//! InvoiceStatus enum - workflow status for fiscal invoices

use crate::FiscalError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Workflow status for fiscal invoices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    /// Invoice created but not yet emitted
    Draft,
    /// Invoice emitted and fiscally valid
    Emitted,
    /// Invoice voided (nullified with a replacement)
    Voided,
    /// Invoice cancelled before emission
    Cancelled,
}

impl InvoiceStatus {
    /// Returns all available invoice statuses
    pub fn all() -> &'static [InvoiceStatus] {
        &[
            InvoiceStatus::Draft,
            InvoiceStatus::Emitted,
            InvoiceStatus::Voided,
            InvoiceStatus::Cancelled,
        ]
    }

    /// Returns true if the invoice can be voided
    pub fn can_void(&self) -> bool {
        matches!(self, InvoiceStatus::Emitted)
    }

    /// Returns true if the invoice is in a final state
    pub fn is_final(&self) -> bool {
        matches!(self, InvoiceStatus::Voided | InvoiceStatus::Cancelled)
    }

    /// Validates transition from current status to new status
    pub fn can_transition_to(&self, new_status: InvoiceStatus) -> bool {
        match (self, new_status) {
            // From Draft
            (InvoiceStatus::Draft, InvoiceStatus::Emitted) => true,
            (InvoiceStatus::Draft, InvoiceStatus::Cancelled) => true,
            // From Emitted
            (InvoiceStatus::Emitted, InvoiceStatus::Voided) => true,
            // All other transitions are invalid
            _ => false,
        }
    }
}

impl FromStr for InvoiceStatus {
    type Err = FiscalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(InvoiceStatus::Draft),
            "emitted" => Ok(InvoiceStatus::Emitted),
            "voided" | "void" => Ok(InvoiceStatus::Voided),
            "cancelled" | "canceled" => Ok(InvoiceStatus::Cancelled),
            _ => Err(FiscalError::InvalidInvoiceStatus),
        }
    }
}

impl fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvoiceStatus::Draft => write!(f, "draft"),
            InvoiceStatus::Emitted => write!(f, "emitted"),
            InvoiceStatus::Voided => write!(f, "voided"),
            InvoiceStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            InvoiceStatus::from_str("draft").unwrap(),
            InvoiceStatus::Draft
        );
        assert_eq!(
            InvoiceStatus::from_str("emitted").unwrap(),
            InvoiceStatus::Emitted
        );
        assert_eq!(
            InvoiceStatus::from_str("voided").unwrap(),
            InvoiceStatus::Voided
        );
        assert_eq!(
            InvoiceStatus::from_str("void").unwrap(),
            InvoiceStatus::Voided
        );
        assert_eq!(
            InvoiceStatus::from_str("cancelled").unwrap(),
            InvoiceStatus::Cancelled
        );
        assert_eq!(
            InvoiceStatus::from_str("canceled").unwrap(),
            InvoiceStatus::Cancelled
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(InvoiceStatus::Draft.to_string(), "draft");
        assert_eq!(InvoiceStatus::Emitted.to_string(), "emitted");
        assert_eq!(InvoiceStatus::Voided.to_string(), "voided");
        assert_eq!(InvoiceStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_workflow_states() {
        // Draft state
        assert!(!InvoiceStatus::Draft.can_void());
        assert!(!InvoiceStatus::Draft.is_final());

        // Emitted state
        assert!(InvoiceStatus::Emitted.can_void());
        assert!(!InvoiceStatus::Emitted.is_final());

        // Final states
        assert!(InvoiceStatus::Voided.is_final());
        assert!(InvoiceStatus::Cancelled.is_final());
    }

    #[test]
    fn test_valid_transitions() {
        // From Draft
        assert!(InvoiceStatus::Draft.can_transition_to(InvoiceStatus::Emitted));
        assert!(InvoiceStatus::Draft.can_transition_to(InvoiceStatus::Cancelled));
        assert!(!InvoiceStatus::Draft.can_transition_to(InvoiceStatus::Voided));

        // From Emitted
        assert!(InvoiceStatus::Emitted.can_transition_to(InvoiceStatus::Voided));
        assert!(!InvoiceStatus::Emitted.can_transition_to(InvoiceStatus::Cancelled));

        // From final states
        assert!(!InvoiceStatus::Voided.can_transition_to(InvoiceStatus::Draft));
        assert!(!InvoiceStatus::Cancelled.can_transition_to(InvoiceStatus::Draft));
    }
}
