//! PaymentStatus enum - status of a payment transaction

use crate::SalesError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Status of a payment transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    /// Payment is pending
    Pending,
    /// Payment completed successfully
    Completed,
    /// Payment failed
    Failed,
    /// Payment fully refunded
    Refunded,
    /// Payment partially refunded
    PartiallyRefunded,
}

impl PaymentStatus {
    /// Returns all available payment statuses
    pub fn all() -> &'static [PaymentStatus] {
        &[
            PaymentStatus::Pending,
            PaymentStatus::Completed,
            PaymentStatus::Failed,
            PaymentStatus::Refunded,
            PaymentStatus::PartiallyRefunded,
        ]
    }

    /// Returns true if the payment is in a final state
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            PaymentStatus::Failed | PaymentStatus::Refunded
        )
    }

    /// Returns true if the payment was successful
    pub fn is_successful(&self) -> bool {
        matches!(
            self,
            PaymentStatus::Completed | PaymentStatus::PartiallyRefunded
        )
    }

    /// Returns true if the payment can be refunded
    pub fn can_refund(&self) -> bool {
        matches!(
            self,
            PaymentStatus::Completed | PaymentStatus::PartiallyRefunded
        )
    }

    /// Validates transition from current status to new status
    pub fn can_transition_to(&self, new_status: PaymentStatus) -> bool {
        match (self, new_status) {
            // From Pending
            (PaymentStatus::Pending, PaymentStatus::Completed) => true,
            (PaymentStatus::Pending, PaymentStatus::Failed) => true,
            // From Completed
            (PaymentStatus::Completed, PaymentStatus::Refunded) => true,
            (PaymentStatus::Completed, PaymentStatus::PartiallyRefunded) => true,
            // From PartiallyRefunded
            (PaymentStatus::PartiallyRefunded, PaymentStatus::Refunded) => true,
            // All other transitions are invalid
            _ => false,
        }
    }
}

impl FromStr for PaymentStatus {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "pending" => Ok(PaymentStatus::Pending),
            "completed" | "complete" | "success" => Ok(PaymentStatus::Completed),
            "failed" | "fail" | "error" => Ok(PaymentStatus::Failed),
            "refunded" | "refund" => Ok(PaymentStatus::Refunded),
            "partially_refunded" | "partiallyrefunded" | "partial_refund" => {
                Ok(PaymentStatus::PartiallyRefunded)
            }
            _ => Err(SalesError::InvalidPaymentStatus),
        }
    }
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "pending"),
            PaymentStatus::Completed => write!(f, "completed"),
            PaymentStatus::Failed => write!(f, "failed"),
            PaymentStatus::Refunded => write!(f, "refunded"),
            PaymentStatus::PartiallyRefunded => write!(f, "partially_refunded"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            PaymentStatus::from_str("pending").unwrap(),
            PaymentStatus::Pending
        );
        assert_eq!(
            PaymentStatus::from_str("completed").unwrap(),
            PaymentStatus::Completed
        );
        assert_eq!(
            PaymentStatus::from_str("success").unwrap(),
            PaymentStatus::Completed
        );
        assert_eq!(
            PaymentStatus::from_str("partially_refunded").unwrap(),
            PaymentStatus::PartiallyRefunded
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(PaymentStatus::Pending.to_string(), "pending");
        assert_eq!(PaymentStatus::Completed.to_string(), "completed");
    }

    #[test]
    fn test_predicates() {
        assert!(!PaymentStatus::Pending.is_final());
        assert!(!PaymentStatus::Pending.is_successful());

        assert!(PaymentStatus::Completed.is_successful());
        assert!(PaymentStatus::Completed.can_refund());

        assert!(PaymentStatus::Failed.is_final());
        assert!(PaymentStatus::Refunded.is_final());
    }

    #[test]
    fn test_valid_transitions() {
        assert!(PaymentStatus::Pending.can_transition_to(PaymentStatus::Completed));
        assert!(PaymentStatus::Pending.can_transition_to(PaymentStatus::Failed));
        assert!(PaymentStatus::Completed.can_transition_to(PaymentStatus::Refunded));
        assert!(PaymentStatus::Completed.can_transition_to(PaymentStatus::PartiallyRefunded));
        assert!(PaymentStatus::PartiallyRefunded.can_transition_to(PaymentStatus::Refunded));
    }
}
