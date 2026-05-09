use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::SubscriptionError;
use crate::domain::entities::DunningAttempt;
use crate::domain::value_objects::{BillingCycleId, DunningAttemptId};

#[async_trait]
pub trait DunningAttemptRepository: Send + Sync {
    async fn save(&self, attempt: &DunningAttempt) -> Result<(), SubscriptionError>;
    async fn update(&self, attempt: &DunningAttempt) -> Result<(), SubscriptionError>;

    async fn find_by_id(
        &self,
        id: DunningAttemptId,
    ) -> Result<Option<DunningAttempt>, SubscriptionError>;

    /// All attempts for a billing cycle, ordered by `attempt_number`.
    async fn find_by_billing_cycle(
        &self,
        billing_cycle_id: BillingCycleId,
    ) -> Result<Vec<DunningAttempt>, SubscriptionError>;

    /// Lookup by the `payments` transaction id stamped on the attempt during
    /// its `process_dunning_attempt` execution. Used by the webhook subscriber.
    async fn find_by_transaction_id(
        &self,
        transaction_id: Uuid,
    ) -> Result<Option<DunningAttempt>, SubscriptionError>;

    /// Pending attempts whose `scheduled_at <= now` and that haven't yet been
    /// fired (no transaction stamped) — fed to the dunning job for execution.
    async fn find_due(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<DunningAttempt>, SubscriptionError>;
}
