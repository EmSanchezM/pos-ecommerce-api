use std::sync::Arc;
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::entities::BankAccount;
use crate::domain::repositories::BankAccountRepository;

pub struct ListBankAccountsUseCase {
    repo: Arc<dyn BankAccountRepository>,
}

impl ListBankAccountsUseCase {
    pub fn new(repo: Arc<dyn BankAccountRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        store_id: Option<Uuid>,
    ) -> Result<Vec<BankAccount>, CashManagementError> {
        self.repo.list(store_id).await
    }
}
