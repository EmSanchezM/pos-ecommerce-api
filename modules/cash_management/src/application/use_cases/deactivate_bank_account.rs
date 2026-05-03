use std::sync::Arc;

use crate::CashManagementError;
use crate::domain::entities::BankAccount;
use crate::domain::repositories::BankAccountRepository;
use crate::domain::value_objects::BankAccountId;

pub struct DeactivateBankAccountUseCase {
    repo: Arc<dyn BankAccountRepository>,
}

impl DeactivateBankAccountUseCase {
    pub fn new(repo: Arc<dyn BankAccountRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: BankAccountId) -> Result<BankAccount, CashManagementError> {
        let mut account = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| CashManagementError::BankAccountNotFound(id.into_uuid()))?;
        account.deactivate();
        self.repo.update(&account).await?;
        account.increment_version();
        Ok(account)
    }
}
