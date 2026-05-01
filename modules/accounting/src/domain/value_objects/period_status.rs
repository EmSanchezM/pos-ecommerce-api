use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::AccountingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodStatus {
    Open,
    Closed,
}

impl fmt::Display for PeriodStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PeriodStatus::Open => "open",
            PeriodStatus::Closed => "closed",
        };
        f.write_str(s)
    }
}

impl FromStr for PeriodStatus {
    type Err = AccountingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            other => Err(AccountingError::InvalidPeriodStatus(other.into())),
        }
    }
}
