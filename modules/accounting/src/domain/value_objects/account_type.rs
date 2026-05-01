//! AccountType — top-level classification used to drive P&L and balance sheet
//! aggregations and to apply normal-balance rules during posting.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::AccountingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

impl AccountType {
    /// Whether this account type's "natural" balance is the debit side.
    /// (Assets and Expenses increase with debits; Liabilities, Equity, Revenue
    /// increase with credits.)
    pub fn is_debit_normal(self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Expense)
    }

    /// Returns true for account types that show on the Profit & Loss
    /// statement (Revenue and Expense).
    pub fn is_pnl(self) -> bool {
        matches!(self, AccountType::Revenue | AccountType::Expense)
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AccountType::Asset => "asset",
            AccountType::Liability => "liability",
            AccountType::Equity => "equity",
            AccountType::Revenue => "revenue",
            AccountType::Expense => "expense",
        };
        f.write_str(s)
    }
}

impl FromStr for AccountType {
    type Err = AccountingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asset" => Ok(Self::Asset),
            "liability" => Ok(Self::Liability),
            "equity" => Ok(Self::Equity),
            "revenue" => Ok(Self::Revenue),
            "expense" => Ok(Self::Expense),
            other => Err(AccountingError::InvalidAccountType(other.into())),
        }
    }
}
