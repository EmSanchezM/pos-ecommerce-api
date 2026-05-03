//! CashDepositStatus — `pending → deposited → reconciled`. `pending` is set
//! when the cash is counted at shift close; `deposited` once it physically
//! reaches the bank (manager confirms); `reconciled` when matched against a
//! `BankTransaction` from the statement.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::CashManagementError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CashDepositStatus {
    Pending,
    Deposited,
    Reconciled,
}

impl CashDepositStatus {
    pub fn can_transition_to(self, next: CashDepositStatus) -> bool {
        use CashDepositStatus::*;
        matches!((self, next), (Pending, Deposited) | (Deposited, Reconciled))
    }
}

impl fmt::Display for CashDepositStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CashDepositStatus::Pending => "pending",
            CashDepositStatus::Deposited => "deposited",
            CashDepositStatus::Reconciled => "reconciled",
        };
        f.write_str(s)
    }
}

impl FromStr for CashDepositStatus {
    type Err = CashManagementError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "deposited" => Ok(Self::Deposited),
            "reconciled" => Ok(Self::Reconciled),
            other => Err(CashManagementError::InvalidDepositStatus(other.into())),
        }
    }
}
