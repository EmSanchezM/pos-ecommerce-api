// PurchaseOrderId value object - unique identifier for purchase orders

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a PurchaseOrder
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PurchaseOrderId(Uuid);

impl PurchaseOrderId {
    /// Creates a new PurchaseOrderId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a PurchaseOrderId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Converts the PurchaseOrderId into its underlying UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for PurchaseOrderId {
    fn default() -> Self {
        Self::new()
    }
}
