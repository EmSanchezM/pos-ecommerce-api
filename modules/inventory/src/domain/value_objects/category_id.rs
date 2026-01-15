// CategoryId value object - unique identifier for product categories

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a ProductCategory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(Uuid);

impl CategoryId {
    /// Creates a new CategoryId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a CategoryId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Converts the CategoryId into its underlying UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for CategoryId {
    fn default() -> Self {
        Self::new()
    }
}
