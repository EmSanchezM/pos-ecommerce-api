use std::sync::Arc;

use crate::CashManagementError;
use crate::application::dtos::StartReconciliationCommand;
use crate::domain::entities::BankReconciliation;
use crate::domain::repositories::{BankAccountRepository, BankReconciliationRepository};

pub struct StartReconciliationUseCase {
    accounts: Arc<dyn BankAccountRepository>,
    repo: Arc<dyn BankReconciliationRepository>,
}

impl StartReconciliationUseCase {
    pub fn new(
        accounts: Arc<dyn BankAccountRepository>,
        repo: Arc<dyn BankReconciliationRepository>,
    ) -> Self {
        Self { accounts, repo }
    }

    pub async fn execute(
        &self,
        cmd: StartReconciliationCommand,
    ) -> Result<BankReconciliation, CashManagementError> {
        let _account = self
            .accounts
            .find_by_id(cmd.bank_account_id)
            .await?
            .ok_or_else(|| {
                CashManagementError::BankAccountNotFound(cmd.bank_account_id.into_uuid())
            })?;

        let recon = BankReconciliation::start(
            cmd.bank_account_id,
            cmd.period_start,
            cmd.period_end,
            cmd.opening_balance,
        )?;
        self.repo.save(&recon).await?;
        Ok(recon)
    }
}
