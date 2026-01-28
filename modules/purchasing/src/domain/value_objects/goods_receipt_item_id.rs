// GoodsReceiptItemId value object - unique identifier for goods receipt items

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a GoodsReceiptItem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoodsReceiptItemId(Uuid);

impl GoodsReceiptItemId {
    /// Creates a new GoodsReceiptItemId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a GoodsReceiptItemId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Converts the GoodsReceiptItemId into its underlying UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for GoodsReceiptItemId {
    fn default() -> Self {
        Self::new()
    }
}
