use std::sync::Arc;

use crate::AccountingError;
use crate::domain::entities::JournalEntry;
use crate::domain::repositories::JournalEntryRepository;
use crate::domain::value_objects::AccountingPeriodId;

pub struct ListJournalEntriesUseCase {
    repo: Arc<dyn JournalEntryRepository>,
}

impl ListJournalEntriesUseCase {
    pub fn new(repo: Arc<dyn JournalEntryRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        period_id: AccountingPeriodId,
    ) -> Result<Vec<JournalEntry>, AccountingError> {
        self.repo.list_by_period(period_id).await
    }
}
