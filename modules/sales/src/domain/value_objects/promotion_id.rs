//! PromotionId value object - unique identifier for promotions

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a Promotion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PromotionId(Uuid);

impl PromotionId {
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

impl Default for PromotionId {
    fn default() -> Self {
        Self::new()
    }
}
