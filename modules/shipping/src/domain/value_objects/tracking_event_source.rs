use crate::ShippingError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackingEventSource {
    System,
    Driver,
    Provider,
    Webhook,
    Manual,
}

impl FromStr for TrackingEventSource {
    type Err = ShippingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "system" => Ok(Self::System),
            "driver" => Ok(Self::Driver),
            "provider" => Ok(Self::Provider),
            "webhook" => Ok(Self::Webhook),
            "manual" => Ok(Self::Manual),
            _ => Err(ShippingError::InvalidTrackingSource),
        }
    }
}

impl fmt::Display for TrackingEventSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::System => "system",
            Self::Driver => "driver",
            Self::Provider => "provider",
            Self::Webhook => "webhook",
            Self::Manual => "manual",
        };
        f.write_str(s)
    }
}
