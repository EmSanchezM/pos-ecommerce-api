//! RewardType — the kind of value a redeemed `Reward` produces. The actual
//! application of the reward to a sale lives in the storefront / POS — this
//! module only owns the points-and-voucher accounting.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::LoyaltyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewardType {
    /// Fixed currency discount; `value` is the amount (e.g. L 50).
    DiscountAmount,
    /// Percentage discount; `value` is the percent (e.g. 10 for 10 %).
    DiscountPercent,
    /// Free product/item; `value` carries the SKU or product id reference
    /// when the storefront wants to enforce it. v1 stores it free-form.
    FreeProduct,
}

impl fmt::Display for RewardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RewardType::DiscountAmount => "discount_amount",
            RewardType::DiscountPercent => "discount_percent",
            RewardType::FreeProduct => "free_product",
        };
        f.write_str(s)
    }
}

impl FromStr for RewardType {
    type Err = LoyaltyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "discount_amount" => Ok(Self::DiscountAmount),
            "discount_percent" => Ok(Self::DiscountPercent),
            "free_product" => Ok(Self::FreeProduct),
            other => Err(LoyaltyError::InvalidRewardType(other.into())),
        }
    }
}
