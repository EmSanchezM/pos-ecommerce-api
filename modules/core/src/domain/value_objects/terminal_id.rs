// TerminalId value object - full implementation in task 2.1

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a Terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerminalId(Uuid);

impl TerminalId {
    /// Creates a new TerminalId with a random UUID
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a TerminalId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Converts the TerminalId into its underlying UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for TerminalId {
    fn default() -> Self {
        Self::new()
    }
}
