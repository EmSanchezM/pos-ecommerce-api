//! `record_payment_outcome` — called by the events subscriber when a
//! `payment.confirmed` / `payment.rejected` webhook lands.
//!
//! Resolves the gateway `transaction_id` to either a `BillingCycle` (first
//! attempt) or a `DunningAttempt` (retry), and applies the side-effects:
//!
//! - Confirmed: cycle → `Paid`, subscription → `Active` (if `PastDue`),
//!   dunning attempt → `Succeeded`.
//! - Rejected: cycle → `Failed` (and schedules 3 dunning attempts at +1d/+3d/+7d),
//!   or dunning attempt → `Failed`. After the 3rd dunning failure on a cycle,
//!   the subscription transitions to `PastDue`.

use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::SubscriptionError;
use crate::domain::entities::DunningAttempt;
use crate::domain::repositories::{
    BillingCycleRepository, DunningAttemptRepository, SubscriptionRepository,
};
use crate::domain::value_objects::{DunningOutcome, SubscriptionStatus};

const DUNNING_DELAYS_DAYS: [i64; 3] = [1, 3, 7];

#[derive(Debug, Clone, Copy)]
pub enum PaymentOutcome<'a> {
    Confirmed,
    Rejected { reason: &'a str },
}

pub struct RecordPaymentOutcomeUseCase {
    sub_repo: Arc<dyn SubscriptionRepository>,
    cycle_repo: Arc<dyn BillingCycleRepository>,
    dunning_repo: Arc<dyn DunningAttemptRepository>,
}

impl RecordPaymentOutcomeUseCase {
    pub fn new(
        sub_repo: Arc<dyn SubscriptionRepository>,
        cycle_repo: Arc<dyn BillingCycleRepository>,
        dunning_repo: Arc<dyn DunningAttemptRepository>,
    ) -> Self {
        Self {
            sub_repo,
            cycle_repo,
            dunning_repo,
        }
    }

    /// `transaction_id` is the gateway-side id; we look it up against both
    /// `billing_cycles.transaction_id` and `dunning_attempts.transaction_id`.
    /// A miss in both means the webhook is for a non-subscription charge —
    /// returns `Ok(false)` so the subscriber can ack quietly.
    pub async fn execute(
        &self,
        transaction_id: Uuid,
        outcome: PaymentOutcome<'_>,
    ) -> Result<bool, SubscriptionError> {
        if let Some(attempt) = self
            .dunning_repo
            .find_by_transaction_id(transaction_id)
            .await?
        {
            self.handle_dunning_outcome(attempt, outcome).await?;
            return Ok(true);
        }
        if let Some(cycle) = self
            .cycle_repo
            .find_by_transaction_id(transaction_id)
            .await?
        {
            self.handle_cycle_outcome(cycle, outcome).await?;
            return Ok(true);
        }
        Ok(false)
    }

    async fn handle_cycle_outcome(
        &self,
        mut cycle: crate::domain::entities::BillingCycle,
        outcome: PaymentOutcome<'_>,
    ) -> Result<(), SubscriptionError> {
        let now = Utc::now();
        match outcome {
            PaymentOutcome::Confirmed => {
                cycle.mark_paid(now)?;
                self.cycle_repo.update(&cycle).await?;
                self.try_resume_subscription(cycle.subscription_id())
                    .await?;
            }
            PaymentOutcome::Rejected { reason } => {
                cycle.mark_failed(reason.to_string(), now)?;
                self.cycle_repo.update(&cycle).await?;
                self.schedule_dunning(cycle.id(), now).await?;
            }
        }
        Ok(())
    }

    async fn handle_dunning_outcome(
        &self,
        mut attempt: DunningAttempt,
        outcome: PaymentOutcome<'_>,
    ) -> Result<(), SubscriptionError> {
        let now = Utc::now();
        let transaction_id = attempt.transaction_id().ok_or_else(|| {
            SubscriptionError::Validation(
                "dunning attempt has no transaction_id stamped".to_string(),
            )
        })?;

        match outcome {
            PaymentOutcome::Confirmed => {
                attempt.mark_succeeded(transaction_id, now)?;
                self.dunning_repo.update(&attempt).await?;
                // The cycle this attempt belonged to also transitions to
                // Paid; the subscription resumes if it was PastDue.
                if let Some(mut cycle) = self
                    .cycle_repo
                    .find_by_id(attempt.billing_cycle_id())
                    .await?
                {
                    cycle.mark_paid(now)?;
                    self.cycle_repo.update(&cycle).await?;
                    self.try_resume_subscription(cycle.subscription_id())
                        .await?;
                }
            }
            PaymentOutcome::Rejected { reason } => {
                attempt.mark_failed(reason.to_string(), transaction_id, now)?;
                self.dunning_repo.update(&attempt).await?;

                // If this was the 3rd (and final) attempt, the parent
                // subscription transitions to PastDue.
                let attempts = self
                    .dunning_repo
                    .find_by_billing_cycle(attempt.billing_cycle_id())
                    .await?;
                let exhausted = attempts.iter().all(|a| {
                    matches!(
                        a.outcome(),
                        DunningOutcome::Failed | DunningOutcome::Skipped
                    )
                });
                let final_attempt = attempts.len() >= DUNNING_DELAYS_DAYS.len() && exhausted;
                if final_attempt
                    && let Some(cycle) = self
                        .cycle_repo
                        .find_by_id(attempt.billing_cycle_id())
                        .await?
                    && let Some(mut sub) = self.sub_repo.find_by_id(cycle.subscription_id()).await?
                    && (sub.status() == SubscriptionStatus::Active
                        || sub.status() == SubscriptionStatus::Trialing)
                {
                    sub.mark_past_due()?;
                    self.sub_repo.update_with_version(&sub).await?;
                    // TODO(events): publish `subscription.past_due`.
                }
            }
        }
        Ok(())
    }

    async fn schedule_dunning(
        &self,
        cycle_id: crate::domain::value_objects::BillingCycleId,
        anchor: chrono::DateTime<Utc>,
    ) -> Result<(), SubscriptionError> {
        for (idx, delay) in DUNNING_DELAYS_DAYS.iter().enumerate() {
            let attempt = DunningAttempt::schedule(
                cycle_id,
                (idx as i16) + 1,
                anchor + Duration::days(*delay),
            );
            self.dunning_repo.save(&attempt).await?;
        }
        Ok(())
    }

    async fn try_resume_subscription(
        &self,
        subscription_id: crate::domain::value_objects::SubscriptionId,
    ) -> Result<(), SubscriptionError> {
        let Some(mut sub) = self.sub_repo.find_by_id(subscription_id).await? else {
            return Ok(());
        };
        if sub.status() == SubscriptionStatus::PastDue {
            sub.resume_active()?;
            self.sub_repo.update_with_version(&sub).await?;
        }
        Ok(())
    }
}
