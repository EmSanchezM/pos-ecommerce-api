use std::sync::Arc;

use crate::AccountingError;
use crate::domain::entities::AccountingPeriod;
use crate::domain::repositories::AccountingPeriodRepository;
use crate::domain::value_objects::AccountingPeriodId;

pub struct ClosePeriodUseCase {
    repo: Arc<dyn AccountingPeriodRepository>,
}

impl ClosePeriodUseCase {
    pub fn new(repo: Arc<dyn AccountingPeriodRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        id: AccountingPeriodId,
    ) -> Result<AccountingPeriod, AccountingError> {
        let mut period = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AccountingError::PeriodNotFound(id.into_uuid()))?;
        period.close()?;
        self.repo.update(&period).await?;
        Ok(period)
    }
}
