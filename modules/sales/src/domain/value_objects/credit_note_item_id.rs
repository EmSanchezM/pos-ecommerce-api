//! CreditNoteItemId value object - unique identifier for credit note line items

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Unique identifier for a CreditNoteItem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CreditNoteItemId(Uuid);

impl CreditNoteItemId {
    /// Creates a new CreditNoteItemId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a CreditNoteItemId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Converts the CreditNoteItemId into its underlying UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for CreditNoteItemId {
    fn default() -> Self {
        Self::new()
    }
}
