use std::sync::Arc;

use crate::CashManagementError;
use crate::application::dtos::UpdateBankAccountCommand;
use crate::domain::entities::BankAccount;
use crate::domain::repositories::BankAccountRepository;
use crate::domain::value_objects::BankAccountId;

pub struct UpdateBankAccountUseCase {
    repo: Arc<dyn BankAccountRepository>,
}

impl UpdateBankAccountUseCase {
    pub fn new(repo: Arc<dyn BankAccountRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        id: BankAccountId,
        cmd: UpdateBankAccountCommand,
    ) -> Result<BankAccount, CashManagementError> {
        let mut account = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| CashManagementError::BankAccountNotFound(id.into_uuid()))?;

        if let Some(name) = cmd.bank_name {
            account.rename(name);
        }
        if let Some(t) = cmd.account_type {
            account.change_type(t);
        }
        self.repo.update(&account).await?;
        // The repo bumped version + 1 in the UPDATE; mirror it in memory so
        // the caller's next edit uses the new version.
        account.increment_version();
        Ok(account)
    }
}
