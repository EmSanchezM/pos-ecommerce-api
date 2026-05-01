//! PayoutStatus enum

use crate::PaymentsError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Lifecycle status for a gateway payout (settlement)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutStatus {
    Pending,
    InTransit,
    Paid,
    Failed,
    Cancelled,
}

impl FromStr for PayoutStatus {
    type Err = PaymentsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "pending" => Ok(PayoutStatus::Pending),
            "in_transit" | "intransit" => Ok(PayoutStatus::InTransit),
            "paid" => Ok(PayoutStatus::Paid),
            "failed" => Ok(PayoutStatus::Failed),
            "cancelled" | "canceled" => Ok(PayoutStatus::Cancelled),
            _ => Err(PaymentsError::InvalidPayoutStatus),
        }
    }
}

impl fmt::Display for PayoutStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PayoutStatus::Pending => write!(f, "pending"),
            PayoutStatus::InTransit => write!(f, "in_transit"),
            PayoutStatus::Paid => write!(f, "paid"),
            PayoutStatus::Failed => write!(f, "failed"),
            PayoutStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}
