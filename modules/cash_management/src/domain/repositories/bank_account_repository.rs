use async_trait::async_trait;
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::entities::BankAccount;
use crate::domain::value_objects::BankAccountId;

#[async_trait]
pub trait BankAccountRepository: Send + Sync {
    async fn save(&self, account: &BankAccount) -> Result<(), CashManagementError>;

    /// Update with optimistic locking on `version`.
    async fn update(&self, account: &BankAccount) -> Result<(), CashManagementError>;

    async fn find_by_id(
        &self,
        id: BankAccountId,
    ) -> Result<Option<BankAccount>, CashManagementError>;

    async fn find_by_account_number(
        &self,
        account_number: &str,
    ) -> Result<Option<BankAccount>, CashManagementError>;

    async fn list(&self, store_id: Option<Uuid>) -> Result<Vec<BankAccount>, CashManagementError>;
}
