//! BankTransactionType — coarse classification of statement lines. The amount
//! sign on the entity follows accounting convention (positive = inflow,
//! negative = outflow); the type field disambiguates "why".

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::CashManagementError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BankTransactionType {
    Deposit,
    Withdrawal,
    Fee,
    Interest,
    TransferIn,
    TransferOut,
    Adjustment,
}

impl BankTransactionType {
    /// True when the type represents money entering the account.
    pub fn is_inflow(self) -> bool {
        matches!(
            self,
            BankTransactionType::Deposit
                | BankTransactionType::Interest
                | BankTransactionType::TransferIn
        )
    }
}

impl fmt::Display for BankTransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BankTransactionType::Deposit => "deposit",
            BankTransactionType::Withdrawal => "withdrawal",
            BankTransactionType::Fee => "fee",
            BankTransactionType::Interest => "interest",
            BankTransactionType::TransferIn => "transfer_in",
            BankTransactionType::TransferOut => "transfer_out",
            BankTransactionType::Adjustment => "adjustment",
        };
        f.write_str(s)
    }
}

impl FromStr for BankTransactionType {
    type Err = CashManagementError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deposit" => Ok(Self::Deposit),
            "withdrawal" => Ok(Self::Withdrawal),
            "fee" => Ok(Self::Fee),
            "interest" => Ok(Self::Interest),
            "transfer_in" => Ok(Self::TransferIn),
            "transfer_out" => Ok(Self::TransferOut),
            "adjustment" => Ok(Self::Adjustment),
            other => Err(CashManagementError::InvalidTransactionType(other.into())),
        }
    }
}
