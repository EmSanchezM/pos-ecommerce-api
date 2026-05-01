use std::sync::Arc;

use crate::AccountingError;
use crate::domain::entities::JournalEntry;
use crate::domain::repositories::JournalEntryRepository;
use crate::domain::value_objects::JournalEntryId;

pub struct GetJournalEntryUseCase {
    repo: Arc<dyn JournalEntryRepository>,
}

impl GetJournalEntryUseCase {
    pub fn new(repo: Arc<dyn JournalEntryRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: JournalEntryId) -> Result<JournalEntry, AccountingError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AccountingError::JournalEntryNotFound(id.into_uuid()))
    }
}
