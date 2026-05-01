use std::sync::Arc;

use crate::AccountingError;
use crate::application::dtos::OpenPeriodCommand;
use crate::domain::entities::AccountingPeriod;
use crate::domain::repositories::AccountingPeriodRepository;

pub struct OpenPeriodUseCase {
    repo: Arc<dyn AccountingPeriodRepository>,
}

impl OpenPeriodUseCase {
    pub fn new(repo: Arc<dyn AccountingPeriodRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        cmd: OpenPeriodCommand,
    ) -> Result<AccountingPeriod, AccountingError> {
        let period =
            AccountingPeriod::create(cmd.name, cmd.fiscal_year, cmd.starts_at, cmd.ends_at)?;
        self.repo.save(&period).await?;
        Ok(period)
    }
}
