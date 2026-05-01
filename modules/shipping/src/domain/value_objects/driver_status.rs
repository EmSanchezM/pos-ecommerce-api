use crate::ShippingError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriverStatus {
    Offline,
    Available,
    Busy,
}

impl FromStr for DriverStatus {
    type Err = ShippingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "offline" => Ok(Self::Offline),
            "available" => Ok(Self::Available),
            "busy" => Ok(Self::Busy),
            _ => Err(ShippingError::InvalidDriverStatus),
        }
    }
}

impl fmt::Display for DriverStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Offline => "offline",
            Self::Available => "available",
            Self::Busy => "busy",
        };
        f.write_str(s)
    }
}
