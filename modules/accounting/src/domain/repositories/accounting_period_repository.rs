use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::AccountingError;
use crate::domain::entities::AccountingPeriod;
use crate::domain::value_objects::AccountingPeriodId;

#[async_trait]
pub trait AccountingPeriodRepository: Send + Sync {
    async fn save(&self, period: &AccountingPeriod) -> Result<(), AccountingError>;
    async fn update(&self, period: &AccountingPeriod) -> Result<(), AccountingError>;
    async fn find_by_id(
        &self,
        id: AccountingPeriodId,
    ) -> Result<Option<AccountingPeriod>, AccountingError>;
    /// Returns the period whose [starts_at, ends_at) range contains `at`.
    async fn find_containing(
        &self,
        at: DateTime<Utc>,
    ) -> Result<Option<AccountingPeriod>, AccountingError>;
    async fn list(&self) -> Result<Vec<AccountingPeriod>, AccountingError>;
}
