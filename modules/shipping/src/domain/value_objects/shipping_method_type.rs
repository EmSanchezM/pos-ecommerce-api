use crate::ShippingError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Operational scenario a method falls into.
///
/// `StorePickup`, `OwnDelivery` and `ExternalDelivery` are the three primary
/// flows; the others are generic carrier-agnostic labels for the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShippingMethodType {
    StorePickup,
    OwnDelivery,
    ExternalDelivery,
    Standard,
    Express,
    SameDay,
    FreeShipping,
}

impl ShippingMethodType {
    pub fn requires_driver(&self) -> bool {
        matches!(self, Self::OwnDelivery)
    }

    pub fn requires_provider(&self) -> bool {
        matches!(self, Self::ExternalDelivery)
    }

    pub fn is_pickup(&self) -> bool {
        matches!(self, Self::StorePickup)
    }
}

impl FromStr for ShippingMethodType {
    type Err = ShippingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "store_pickup" | "pickup" | "retiro" => Ok(Self::StorePickup),
            "own_delivery" | "in_house" => Ok(Self::OwnDelivery),
            "external_delivery" | "third_party" | "external" => Ok(Self::ExternalDelivery),
            "standard" => Ok(Self::Standard),
            "express" => Ok(Self::Express),
            "same_day" | "sameday" => Ok(Self::SameDay),
            "free_shipping" | "free" => Ok(Self::FreeShipping),
            _ => Err(ShippingError::InvalidMethodType),
        }
    }
}

impl fmt::Display for ShippingMethodType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StorePickup => write!(f, "store_pickup"),
            Self::OwnDelivery => write!(f, "own_delivery"),
            Self::ExternalDelivery => write!(f, "external_delivery"),
            Self::Standard => write!(f, "standard"),
            Self::Express => write!(f, "express"),
            Self::SameDay => write!(f, "same_day"),
            Self::FreeShipping => write!(f, "free_shipping"),
        }
    }
}
