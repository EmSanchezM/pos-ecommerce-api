use async_trait::async_trait;

use crate::AccountingError;
use crate::domain::entities::Account;
use crate::domain::value_objects::{AccountId, AccountType};

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn save(&self, account: &Account) -> Result<(), AccountingError>;
    async fn update(&self, account: &Account) -> Result<(), AccountingError>;
    async fn find_by_id(&self, id: AccountId) -> Result<Option<Account>, AccountingError>;
    async fn find_by_code(&self, code: &str) -> Result<Option<Account>, AccountingError>;
    async fn list(&self) -> Result<Vec<Account>, AccountingError>;
    async fn list_by_type(
        &self,
        account_type: AccountType,
    ) -> Result<Vec<Account>, AccountingError>;
}
