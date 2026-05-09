use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::SubscriptionError;
use crate::domain::entities::BillingCycle;
use crate::domain::value_objects::{BillingCycleId, SubscriptionId};

#[async_trait]
pub trait BillingCycleRepository: Send + Sync {
    async fn save(&self, cycle: &BillingCycle) -> Result<(), SubscriptionError>;
    async fn update(&self, cycle: &BillingCycle) -> Result<(), SubscriptionError>;
    async fn find_by_id(
        &self,
        id: BillingCycleId,
    ) -> Result<Option<BillingCycle>, SubscriptionError>;

    /// Lookup by the `payments` transaction id stamped on the cycle. Used by
    /// the webhook subscriber to resolve `payment.confirmed`/`payment.rejected`
    /// events back to the originating cycle.
    async fn find_by_transaction_id(
        &self,
        transaction_id: Uuid,
    ) -> Result<Option<BillingCycle>, SubscriptionError>;

    /// Paginated history of cycles for a subscription, newest first
    /// (`period_end DESC`).
    async fn find_by_subscription(
        &self,
        subscription_id: SubscriptionId,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<BillingCycle>, i64), SubscriptionError>;

    /// Cycles still `Pending` whose `period_start <= now` — fed to the
    /// billing job to generate the invoice + transaction.
    async fn find_pending_due(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<BillingCycle>, SubscriptionError>;
}
