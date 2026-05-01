use std::sync::Arc;

use crate::AccountingError;
use crate::domain::entities::AccountingPeriod;
use crate::domain::repositories::AccountingPeriodRepository;

pub struct ListPeriodsUseCase {
    repo: Arc<dyn AccountingPeriodRepository>,
}

impl ListPeriodsUseCase {
    pub fn new(repo: Arc<dyn AccountingPeriodRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<AccountingPeriod>, AccountingError> {
        self.repo.list().await
    }
}
