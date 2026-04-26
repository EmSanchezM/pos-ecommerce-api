//! TransactionStatus enum

use crate::PaymentsError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Lifecycle status for a payment transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Cancelled,
    Refunded,
    PartiallyRefunded,
}

impl TransactionStatus {
    /// Returns true when a charge can be refunded from this status
    pub fn can_refund(&self) -> bool {
        matches!(
            self,
            TransactionStatus::Succeeded | TransactionStatus::PartiallyRefunded
        )
    }

    /// Returns true when the transaction is in a final state
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            TransactionStatus::Succeeded
                | TransactionStatus::Failed
                | TransactionStatus::Cancelled
                | TransactionStatus::Refunded
        )
    }
}

impl FromStr for TransactionStatus {
    type Err = PaymentsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "pending" => Ok(TransactionStatus::Pending),
            "processing" => Ok(TransactionStatus::Processing),
            "succeeded" | "success" => Ok(TransactionStatus::Succeeded),
            "failed" | "failure" => Ok(TransactionStatus::Failed),
            "cancelled" | "canceled" => Ok(TransactionStatus::Cancelled),
            "refunded" => Ok(TransactionStatus::Refunded),
            "partially_refunded" | "partiallyrefunded" => Ok(TransactionStatus::PartiallyRefunded),
            _ => Err(PaymentsError::InvalidTransactionStatus),
        }
    }
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Processing => write!(f, "processing"),
            TransactionStatus::Succeeded => write!(f, "succeeded"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Cancelled => write!(f, "cancelled"),
            TransactionStatus::Refunded => write!(f, "refunded"),
            TransactionStatus::PartiallyRefunded => write!(f, "partially_refunded"),
        }
    }
}
