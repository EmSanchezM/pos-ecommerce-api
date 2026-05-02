//! BankAccountType — checking / savings / other. Drives report grouping but
//! has no behaviour today.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::CashManagementError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BankAccountType {
    Checking,
    Savings,
    Other,
}

impl fmt::Display for BankAccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BankAccountType::Checking => "checking",
            BankAccountType::Savings => "savings",
            BankAccountType::Other => "other",
        };
        f.write_str(s)
    }
}

impl FromStr for BankAccountType {
    type Err = CashManagementError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "checking" => Ok(Self::Checking),
            "savings" => Ok(Self::Savings),
            "other" => Ok(Self::Other),
            other => Err(CashManagementError::InvalidAccountType(other.into())),
        }
    }
}
