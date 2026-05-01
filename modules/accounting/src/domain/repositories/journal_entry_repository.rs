use async_trait::async_trait;

use crate::AccountingError;
use crate::domain::entities::JournalEntry;
use crate::domain::value_objects::{AccountingPeriodId, JournalEntryId};

#[async_trait]
pub trait JournalEntryRepository: Send + Sync {
    /// Persist a new entry along with its lines in a single transaction.
    async fn save(&self, entry: &JournalEntry) -> Result<(), AccountingError>;

    /// Update header status fields (status, posted_at). Lines are immutable
    /// after creation to preserve audit history; corrections require a new
    /// reversing entry.
    async fn update_status(&self, entry: &JournalEntry) -> Result<(), AccountingError>;

    async fn find_by_id(&self, id: JournalEntryId)
    -> Result<Option<JournalEntry>, AccountingError>;

    async fn list_by_period(
        &self,
        period_id: AccountingPeriodId,
    ) -> Result<Vec<JournalEntry>, AccountingError>;

    /// Returns the next available entry_number for the period (max + 1).
    async fn next_entry_number(
        &self,
        period_id: AccountingPeriodId,
    ) -> Result<i64, AccountingError>;
}
