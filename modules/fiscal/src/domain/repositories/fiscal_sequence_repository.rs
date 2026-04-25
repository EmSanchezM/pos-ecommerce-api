//! FiscalSequence repository trait

use async_trait::async_trait;

use serde::{Deserialize, Serialize};

use crate::FiscalError;
use crate::domain::entities::FiscalSequence;
use crate::domain::value_objects::FiscalSequenceId;
use identity::StoreId;
use pos_core::TerminalId;

/// Result of atomically incrementing a fiscal sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextSequenceResult {
    /// The formatted invoice number
    pub invoice_number: String,
    /// The CAI number associated with the sequence
    pub cai_number: String,
    /// The prefix used for the sequence
    pub prefix: String,
    /// The raw sequence number that was assigned
    pub sequence_number: i64,
}

/// Repository trait for FiscalSequence persistence
#[async_trait]
pub trait FiscalSequenceRepository: Send + Sync {
    /// Saves a new fiscal sequence
    async fn save(&self, seq: &FiscalSequence) -> Result<(), FiscalError>;

    /// Finds the active fiscal sequence for a store and terminal
    async fn find_active(
        &self,
        store_id: StoreId,
        terminal_id: TerminalId,
    ) -> Result<Option<FiscalSequence>, FiscalError>;

    /// Atomically increments the sequence and returns the formatted number
    async fn increment_and_get(&self, id: FiscalSequenceId) -> Result<String, FiscalError>;

    /// Updates an existing fiscal sequence
    async fn update(&self, seq: &FiscalSequence) -> Result<(), FiscalError>;
}
