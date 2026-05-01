use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageStorageProviderId(Uuid);

impl ImageStorageProviderId {
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

impl Default for ImageStorageProviderId {
    fn default() -> Self {
        Self::new()
    }
}
