//! AccountingReportRepository — cross-account aggregates against
//! `journal_lines` for posted entries within a period. Implementations run
//! raw SQL so the analytics layer stays decoupled from persistence details.

use async_trait::async_trait;
use uuid::Uuid;

use crate::AccountingError;
use crate::domain::entities::ProfitAndLossLine;
use crate::domain::value_objects::AccountingPeriodId;

#[async_trait]
pub trait AccountingReportRepository: Send + Sync {
    async fn profit_and_loss(
        &self,
        period_id: AccountingPeriodId,
        store_id: Option<Uuid>,
    ) -> Result<Vec<ProfitAndLossLine>, AccountingError>;
}
