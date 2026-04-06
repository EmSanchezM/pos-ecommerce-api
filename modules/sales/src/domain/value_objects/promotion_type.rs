//! PromotionType value object

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::SalesError;

/// Type of promotion/discount
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionType {
    /// Percentage discount off the total
    Percentage,
    /// Fixed amount discount
    FixedAmount,
    /// Buy X items, get Y items free or discounted
    BuyXGetY,
}

impl fmt::Display for PromotionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Percentage => write!(f, "percentage"),
            Self::FixedAmount => write!(f, "fixed_amount"),
            Self::BuyXGetY => write!(f, "buy_x_get_y"),
        }
    }
}

impl FromStr for PromotionType {
    type Err = SalesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "percentage" => Ok(Self::Percentage),
            "fixed_amount" => Ok(Self::FixedAmount),
            "buy_x_get_y" => Ok(Self::BuyXGetY),
            _ => Err(SalesError::InvalidPromotionType),
        }
    }
}
