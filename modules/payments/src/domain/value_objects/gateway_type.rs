//! GatewayType enum - supported payment gateway providers

use crate::PaymentsError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported payment gateway providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatewayType {
    Stripe,
    PayPal,
    BacCredomatic,
    Ficohsa,
    Manual,
}

impl GatewayType {
    pub fn all() -> &'static [GatewayType] {
        &[
            GatewayType::Stripe,
            GatewayType::PayPal,
            GatewayType::BacCredomatic,
            GatewayType::Ficohsa,
            GatewayType::Manual,
        ]
    }

    /// Returns true when the gateway speaks to a remote provider
    pub fn requires_remote_call(&self) -> bool {
        !matches!(self, GatewayType::Manual)
    }
}

impl FromStr for GatewayType {
    type Err = PaymentsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "stripe" => Ok(GatewayType::Stripe),
            "paypal" | "pay_pal" => Ok(GatewayType::PayPal),
            "bac" | "bac_credomatic" | "baccredomatic" => Ok(GatewayType::BacCredomatic),
            "ficohsa" => Ok(GatewayType::Ficohsa),
            "manual" => Ok(GatewayType::Manual),
            _ => Err(PaymentsError::InvalidGatewayType),
        }
    }
}

impl fmt::Display for GatewayType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayType::Stripe => write!(f, "stripe"),
            GatewayType::PayPal => write!(f, "paypal"),
            GatewayType::BacCredomatic => write!(f, "bac_credomatic"),
            GatewayType::Ficohsa => write!(f, "ficohsa"),
            GatewayType::Manual => write!(f, "manual"),
        }
    }
}
