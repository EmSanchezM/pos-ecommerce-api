// GoodsReceiptId value object - unique identifier for goods receipts

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a GoodsReceipt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoodsReceiptId(Uuid);

impl GoodsReceiptId {
    /// Creates a new GoodsReceiptId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a GoodsReceiptId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Converts the GoodsReceiptId into its underlying UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for GoodsReceiptId {
    fn default() -> Self {
        Self::new()
    }
}
