// AdjustmentId value object - unique identifier for stock adjustments

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a StockAdjustment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdjustmentId(Uuid);

impl AdjustmentId {
    /// Creates a new AdjustmentId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates an AdjustmentId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Converts the AdjustmentId into its underlying UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for AdjustmentId {
    fn default() -> Self {
        Self::new()
    }
}
