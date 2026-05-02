use std::sync::Arc;

use crate::CashManagementError;
use crate::domain::entities::BankTransaction;
use crate::domain::repositories::{BankTransactionFilter, BankTransactionRepository};

pub struct ListBankTransactionsUseCase {
    repo: Arc<dyn BankTransactionRepository>,
}

impl ListBankTransactionsUseCase {
    pub fn new(repo: Arc<dyn BankTransactionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        filter: BankTransactionFilter,
    ) -> Result<Vec<BankTransaction>, CashManagementError> {
        self.repo.list(filter).await
    }
}
