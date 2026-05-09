use async_trait::async_trait;
use chrono::{DateTime, Utc};

use tenancy::OrganizationId;

use crate::SubscriptionError;
use crate::domain::entities::Subscription;
use crate::domain::value_objects::SubscriptionId;

#[async_trait]
pub trait SubscriptionRepository: Send + Sync {
    async fn save(&self, subscription: &Subscription) -> Result<(), SubscriptionError>;

    /// Optimistic-lock update.
    ///
    /// Implementations MUST execute the update with the equivalent of
    /// `WHERE id = $id AND version = $entity.version() - 1` and return
    /// `SubscriptionError::OptimisticLockFailed` when zero rows are
    /// affected.
    async fn update_with_version(
        &self,
        subscription: &Subscription,
    ) -> Result<(), SubscriptionError>;

    async fn find_by_id(
        &self,
        id: SubscriptionId,
    ) -> Result<Option<Subscription>, SubscriptionError>;

    /// The single non-`Canceled` subscription for the organization, if any.
    async fn find_active_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Option<Subscription>, SubscriptionError>;

    /// Every subscription the org has ever had, newest first. Useful for
    /// audit/history endpoints.
    async fn find_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Subscription>, SubscriptionError>;

    /// Subscriptions whose `current_period_end <= now` and that are still
    /// `Trialing` or `Active` — fed to the billing job to roll them into a
    /// new period.
    async fn list_due_for_billing(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<Subscription>, SubscriptionError>;

    /// Subscriptions stuck in `PastDue` past the grace period. Fed to the
    /// `tick_past_due_subscriptions` job to cancel + downgrade.
    async fn list_past_due_pending_cancellation(
        &self,
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<Subscription>, SubscriptionError>;
}
