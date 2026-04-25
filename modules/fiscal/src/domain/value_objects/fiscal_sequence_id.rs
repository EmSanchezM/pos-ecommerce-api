//! FiscalSequenceId value object - unique identifier for fiscal sequences

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a FiscalSequence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FiscalSequenceId(Uuid);

impl FiscalSequenceId {
    /// Creates a new FiscalSequenceId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a FiscalSequenceId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Converts the FiscalSequenceId into its underlying UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for FiscalSequenceId {
    fn default() -> Self {
        Self::new()
    }
}
