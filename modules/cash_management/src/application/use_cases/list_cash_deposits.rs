use std::sync::Arc;
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::entities::CashDeposit;
use crate::domain::repositories::CashDepositRepository;
use crate::domain::value_objects::CashDepositStatus;

pub struct ListCashDepositsUseCase {
    repo: Arc<dyn CashDepositRepository>,
}

impl ListCashDepositsUseCase {
    pub fn new(repo: Arc<dyn CashDepositRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        store_id: Option<Uuid>,
        status: Option<CashDepositStatus>,
    ) -> Result<Vec<CashDeposit>, CashManagementError> {
        self.repo.list(store_id, status).await
    }
}
