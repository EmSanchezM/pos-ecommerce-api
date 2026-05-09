//! `tick_past_due_subscriptions` — runs in the periodic billing job.
//!
//! Cancels every subscription that has been stuck in `PastDue` for longer
//! than the grace period (14 days in v1.0). The cancellation is immediate
//! (`canceled_at = now`); a downstream tenancy subscriber can downgrade the
//! org's `OrganizationPlan.tier` back to `free` in response.

use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::SubscriptionError;
use crate::domain::repositories::SubscriptionRepository;

pub const GRACE_PERIOD_DAYS: i64 = 14;

pub struct TickPastDueSubscriptionsUseCase {
    sub_repo: Arc<dyn SubscriptionRepository>,
}

impl TickPastDueSubscriptionsUseCase {
    pub fn new(sub_repo: Arc<dyn SubscriptionRepository>) -> Self {
        Self { sub_repo }
    }

    /// Returns the number of subscriptions canceled in this tick.
    pub async fn execute(&self) -> Result<i64, SubscriptionError> {
        let now = Utc::now();
        let cutoff = now - Duration::days(GRACE_PERIOD_DAYS);
        let subs = self
            .sub_repo
            .list_past_due_pending_cancellation(cutoff)
            .await?;

        let mut canceled = 0i64;
        for mut sub in subs {
            if let Err(e) = sub.cancel(false, now) {
                tracing::warn!(
                    subscription_id = %sub.id().into_uuid(),
                    error = %e,
                    "failed to cancel past-due subscription"
                );
                continue;
            }
            self.sub_repo.update_with_version(&sub).await?;
            canceled += 1;
            // TODO(events): publish `subscription.canceled`.
        }
        Ok(canceled)
    }
}
