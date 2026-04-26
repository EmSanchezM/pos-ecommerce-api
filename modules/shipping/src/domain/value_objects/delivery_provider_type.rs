use crate::ShippingError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryProviderType {
    Hugo,
    PedidosYa,
    UberEats,
    Servientrega,
    Manual,
}

impl FromStr for DeliveryProviderType {
    type Err = ShippingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "hugo" => Ok(Self::Hugo),
            "pedidos_ya" | "pedidosya" => Ok(Self::PedidosYa),
            "uber_eats" | "ubereats" | "uber" => Ok(Self::UberEats),
            "servientrega" => Ok(Self::Servientrega),
            "manual" => Ok(Self::Manual),
            _ => Err(ShippingError::InvalidProviderType),
        }
    }
}

impl fmt::Display for DeliveryProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Hugo => "hugo",
            Self::PedidosYa => "pedidos_ya",
            Self::UberEats => "uber_eats",
            Self::Servientrega => "servientrega",
            Self::Manual => "manual",
        };
        f.write_str(s)
    }
}
