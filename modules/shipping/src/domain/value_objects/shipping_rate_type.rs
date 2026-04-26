use crate::ShippingError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShippingRateType {
    Flat,
    WeightBased,
    OrderBased,
}

impl FromStr for ShippingRateType {
    type Err = ShippingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "flat" => Ok(Self::Flat),
            "weight_based" | "weight" => Ok(Self::WeightBased),
            "order_based" | "order" => Ok(Self::OrderBased),
            _ => Err(ShippingError::InvalidRateType),
        }
    }
}

impl fmt::Display for ShippingRateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flat => write!(f, "flat"),
            Self::WeightBased => write!(f, "weight_based"),
            Self::OrderBased => write!(f, "order_based"),
        }
    }
}
