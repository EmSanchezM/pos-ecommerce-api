// VariantId value object - unique identifier for product variants

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a ProductVariant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VariantId(Uuid);

impl VariantId {
    /// Creates a new VariantId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a VariantId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Converts the VariantId into its underlying UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for VariantId {
    fn default() -> Self {
        Self::new()
    }
}
