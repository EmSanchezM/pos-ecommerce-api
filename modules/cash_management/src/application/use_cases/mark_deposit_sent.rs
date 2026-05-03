use std::sync::Arc;
use uuid::Uuid;

use crate::CashManagementError;
use crate::application::dtos::MarkDepositSentCommand;
use crate::domain::entities::CashDeposit;
use crate::domain::repositories::CashDepositRepository;
use crate::domain::value_objects::CashDepositId;

pub struct MarkDepositSentUseCase {
    repo: Arc<dyn CashDepositRepository>,
}

impl MarkDepositSentUseCase {
    pub fn new(repo: Arc<dyn CashDepositRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        id: CashDepositId,
        actor_id: Uuid,
        cmd: MarkDepositSentCommand,
    ) -> Result<CashDeposit, CashManagementError> {
        let mut deposit = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| CashManagementError::CashDepositNotFound(id.into_uuid()))?;
        deposit.mark_deposited(cmd.deposit_slip_number, actor_id)?;
        self.repo.update(&deposit).await?;
        Ok(deposit)
    }
}
