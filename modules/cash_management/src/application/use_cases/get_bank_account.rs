use std::sync::Arc;

use crate::CashManagementError;
use crate::domain::entities::BankAccount;
use crate::domain::repositories::BankAccountRepository;
use crate::domain::value_objects::BankAccountId;

pub struct GetBankAccountUseCase {
    repo: Arc<dyn BankAccountRepository>,
}

impl GetBankAccountUseCase {
    pub fn new(repo: Arc<dyn BankAccountRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: BankAccountId) -> Result<BankAccount, CashManagementError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| CashManagementError::BankAccountNotFound(id.into_uuid()))
    }
}
