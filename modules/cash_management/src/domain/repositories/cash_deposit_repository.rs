use async_trait::async_trait;
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::entities::CashDeposit;
use crate::domain::value_objects::{CashDepositId, CashDepositStatus};

#[async_trait]
pub trait CashDepositRepository: Send + Sync {
    async fn save(&self, deposit: &CashDeposit) -> Result<(), CashManagementError>;

    async fn update(&self, deposit: &CashDeposit) -> Result<(), CashManagementError>;

    async fn find_by_id(
        &self,
        id: CashDepositId,
    ) -> Result<Option<CashDeposit>, CashManagementError>;

    async fn find_by_shift(
        &self,
        cashier_shift_id: Uuid,
    ) -> Result<Option<CashDeposit>, CashManagementError>;

    async fn list(
        &self,
        store_id: Option<Uuid>,
        status: Option<CashDepositStatus>,
    ) -> Result<Vec<CashDeposit>, CashManagementError>;
}
