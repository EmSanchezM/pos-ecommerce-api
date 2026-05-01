use crate::ShippingError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Lifecycle status. Allowed transitions depend on `ShippingMethodType` —
/// see `Shipment::can_transition_to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipmentStatus {
    Pending,        // initial
    ReadyForPickup, // store_pickup ready
    PickedUp,       // store_pickup completed
    Assigned,       // own_delivery: driver assigned, not yet started
    Dispatched,     // external_delivery: handed off to provider
    InTransit,
    OutForDelivery,
    Delivered,
    Failed,   // delivery attempt failed (recoverable via reschedule)
    Returned, // returned to store
    Cancelled,
    Expired, // store_pickup window passed
}

impl ShipmentStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::PickedUp | Self::Delivered | Self::Returned | Self::Cancelled | Self::Expired
        )
    }

    pub fn is_active(&self) -> bool {
        !self.is_terminal()
    }
}

impl FromStr for ShipmentStatus {
    type Err = ShippingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "pending" => Ok(Self::Pending),
            "ready_for_pickup" => Ok(Self::ReadyForPickup),
            "picked_up" => Ok(Self::PickedUp),
            "assigned" => Ok(Self::Assigned),
            "dispatched" => Ok(Self::Dispatched),
            "in_transit" => Ok(Self::InTransit),
            "out_for_delivery" => Ok(Self::OutForDelivery),
            "delivered" => Ok(Self::Delivered),
            "failed" => Ok(Self::Failed),
            "returned" => Ok(Self::Returned),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            "expired" => Ok(Self::Expired),
            _ => Err(ShippingError::InvalidShipmentStatus),
        }
    }
}

impl fmt::Display for ShipmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::ReadyForPickup => "ready_for_pickup",
            Self::PickedUp => "picked_up",
            Self::Assigned => "assigned",
            Self::Dispatched => "dispatched",
            Self::InTransit => "in_transit",
            Self::OutForDelivery => "out_for_delivery",
            Self::Delivered => "delivered",
            Self::Failed => "failed",
            Self::Returned => "returned",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
        };
        f.write_str(s)
    }
}
