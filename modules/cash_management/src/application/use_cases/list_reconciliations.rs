use std::sync::Arc;

use crate::CashManagementError;
use crate::domain::entities::BankReconciliation;
use crate::domain::repositories::BankReconciliationRepository;
use crate::domain::value_objects::BankAccountId;

pub struct ListReconciliationsUseCase {
    repo: Arc<dyn BankReconciliationRepository>,
}

impl ListReconciliationsUseCase {
    pub fn new(repo: Arc<dyn BankReconciliationRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        bank_account_id: BankAccountId,
    ) -> Result<Vec<BankReconciliation>, CashManagementError> {
        self.repo.list_by_account(bank_account_id).await
    }
}
