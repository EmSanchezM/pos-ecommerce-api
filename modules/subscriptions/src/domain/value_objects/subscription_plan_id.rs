//! Unique identifier for a `SubscriptionPlan`.

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionPlanId(Uuid);

impl SubscriptionPlanId {
    /// Creates a new `SubscriptionPlanId` backed by a UUID v7 (time-ordered).
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SubscriptionPlanId {
    fn default() -> Self {
        Self::new()
    }
}
