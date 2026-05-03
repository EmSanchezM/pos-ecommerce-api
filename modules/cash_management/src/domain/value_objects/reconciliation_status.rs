//! BankReconciliationStatus — workflow state for a periodic reconciliation:
//! `in_progress → completed`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::CashManagementError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BankReconciliationStatus {
    InProgress,
    Completed,
}

impl fmt::Display for BankReconciliationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BankReconciliationStatus::InProgress => "in_progress",
            BankReconciliationStatus::Completed => "completed",
        };
        f.write_str(s)
    }
}

impl FromStr for BankReconciliationStatus {
    type Err = CashManagementError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            other => Err(CashManagementError::InvalidReconciliationStatus(
                other.into(),
            )),
        }
    }
}
