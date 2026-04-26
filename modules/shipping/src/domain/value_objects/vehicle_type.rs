use crate::ShippingError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleType {
    Motorcycle,
    Car,
    Bicycle,
    Pickup,
    Foot,
}

impl FromStr for VehicleType {
    type Err = ShippingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "motorcycle" | "moto" => Ok(Self::Motorcycle),
            "car" | "carro" => Ok(Self::Car),
            "bicycle" | "bici" | "bike" => Ok(Self::Bicycle),
            "pickup" | "truck" => Ok(Self::Pickup),
            "foot" | "walking" => Ok(Self::Foot),
            _ => Err(ShippingError::InvalidVehicleType),
        }
    }
}

impl fmt::Display for VehicleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Motorcycle => "motorcycle",
            Self::Car => "car",
            Self::Bicycle => "bicycle",
            Self::Pickup => "pickup",
            Self::Foot => "foot",
        };
        f.write_str(s)
    }
}
