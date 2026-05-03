//! PointsTransactionType — drives the sign convention on `PointsLedgerEntry`:
//! `Earn` and `Adjustment` (positive variant) are inflows, `Redeem` and
//! `Expire` are outflows. The entity stores the *signed* points value so
//! totals via `SUM(points)` give the correct balance.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::LoyaltyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PointsTransactionType {
    Earn,
    Redeem,
    Expire,
    Adjustment,
}

impl fmt::Display for PointsTransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PointsTransactionType::Earn => "earn",
            PointsTransactionType::Redeem => "redeem",
            PointsTransactionType::Expire => "expire",
            PointsTransactionType::Adjustment => "adjustment",
        };
        f.write_str(s)
    }
}

impl FromStr for PointsTransactionType {
    type Err = LoyaltyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "earn" => Ok(Self::Earn),
            "redeem" => Ok(Self::Redeem),
            "expire" => Ok(Self::Expire),
            "adjustment" => Ok(Self::Adjustment),
            other => Err(LoyaltyError::InvalidTransactionType(other.into())),
        }
    }
}
