use std::sync::Arc;

use crate::AccountingError;
use crate::domain::entities::Account;
use crate::domain::repositories::AccountRepository;
use crate::domain::value_objects::AccountType;

pub struct ListAccountsUseCase {
    repo: Arc<dyn AccountRepository>,
}

impl ListAccountsUseCase {
    pub fn new(repo: Arc<dyn AccountRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        account_type: Option<AccountType>,
    ) -> Result<Vec<Account>, AccountingError> {
        match account_type {
            Some(t) => self.repo.list_by_type(t).await,
            None => self.repo.list().await,
        }
    }
}
