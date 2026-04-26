//! PaymentGatewayId value object - unique identifier for payment gateways

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a PaymentGateway
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaymentGatewayId(Uuid);

impl PaymentGatewayId {
    /// Creates a new PaymentGatewayId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for PaymentGatewayId {
    fn default() -> Self {
        Self::new()
    }
}
