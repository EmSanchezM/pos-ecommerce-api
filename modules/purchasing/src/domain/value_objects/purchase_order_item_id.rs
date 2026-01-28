// PurchaseOrderItemId value object - unique identifier for purchase order items

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a PurchaseOrderItem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PurchaseOrderItemId(Uuid);

impl PurchaseOrderItemId {
    /// Creates a new PurchaseOrderItemId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a PurchaseOrderItemId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Converts the PurchaseOrderItemId into its underlying UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for PurchaseOrderItemId {
    fn default() -> Self {
        Self::new()
    }
}
