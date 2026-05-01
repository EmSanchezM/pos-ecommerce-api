//! TransactionType enum

use crate::PaymentsError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Type of payment transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Charge,
    Refund,
    PartialRefund,
    Void,
}

impl TransactionType {
    pub fn is_refund(&self) -> bool {
        matches!(
            self,
            TransactionType::Refund | TransactionType::PartialRefund
        )
    }
}

impl FromStr for TransactionType {
    type Err = PaymentsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "charge" => Ok(TransactionType::Charge),
            "refund" => Ok(TransactionType::Refund),
            "partial_refund" | "partialrefund" => Ok(TransactionType::PartialRefund),
            "void" => Ok(TransactionType::Void),
            _ => Err(PaymentsError::InvalidTransactionType),
        }
    }
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::Charge => write!(f, "charge"),
            TransactionType::Refund => write!(f, "refund"),
            TransactionType::PartialRefund => write!(f, "partial_refund"),
            TransactionType::Void => write!(f, "void"),
        }
    }
}
