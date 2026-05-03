use async_trait::async_trait;

use crate::CashManagementError;
use crate::domain::entities::BankReconciliation;
use crate::domain::value_objects::{BankAccountId, BankReconciliationId};

#[async_trait]
pub trait BankReconciliationRepository: Send + Sync {
    async fn save(&self, recon: &BankReconciliation) -> Result<(), CashManagementError>;

    async fn update(&self, recon: &BankReconciliation) -> Result<(), CashManagementError>;

    async fn find_by_id(
        &self,
        id: BankReconciliationId,
    ) -> Result<Option<BankReconciliation>, CashManagementError>;

    async fn list_by_account(
        &self,
        bank_account_id: BankAccountId,
    ) -> Result<Vec<BankReconciliation>, CashManagementError>;
}
