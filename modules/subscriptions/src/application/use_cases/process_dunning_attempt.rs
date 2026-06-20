//! `process_dunning_attempt` — invoked by the dunning job for every attempt
//! whose `scheduled_at <= now` and `transaction_id IS NULL` (not yet fired).
//!
//! Calls the payments gateway again, stamps the new `transaction_id` on the
//! attempt, and leaves the outcome `Pending` until the webhook resolves it.

use std::sync::Arc;

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::SubscriptionError;
use crate::domain::entities::DunningAttempt;
use crate::domain::repositories::{
    BillingCycleRepository, DunningAttemptRepository, SubscriptionRepository,
};
use crate::domain::value_objects::DunningAttemptId;
use crate::infrastructure::BillingPaymentGateway;

pub struct ProcessDunningAttemptUseCase {
    sub_repo: Arc<dyn SubscriptionRepository>,
    cycle_repo: Arc<dyn BillingCycleRepository>,
    dunning_repo: Arc<dyn DunningAttemptRepository>,
    payment_gw: Arc<dyn BillingPaymentGateway>,
}

impl ProcessDunningAttemptUseCase {
    pub fn new(
        sub_repo: Arc<dyn SubscriptionRepository>,
        cycle_repo: Arc<dyn BillingCycleRepository>,
        dunning_repo: Arc<dyn DunningAttemptRepository>,
        payment_gw: Arc<dyn BillingPaymentGateway>,
    ) -> Self {
        Self {
            sub_repo,
            cycle_repo,
            dunning_repo,
            payment_gw,
        }
    }

    pub async fn execute(&self, attempt_id: DunningAttemptId) -> Result<(), SubscriptionError> {
        let attempt = self
            .dunning_repo
            .find_by_id(attempt_id)
            .await?
            .ok_or_else(|| SubscriptionError::DunningAttemptNotFound(attempt_id.into_uuid()))?;

        // Already fired? Nothing to do — webhook will resolve it.
        if attempt.transaction_id().is_some() {
            return Ok(());
        }

        let cycle = self
            .cycle_repo
            .find_by_id(attempt.billing_cycle_id())
            .await?
            .ok_or_else(|| {
                SubscriptionError::BillingCycleNotFound(attempt.billing_cycle_id().into_uuid())
            })?;
        let subscription = self
            .sub_repo
            .find_by_id(cycle.subscription_id())
            .await?
            .ok_or_else(|| {
                SubscriptionError::SubscriptionNotFound(cycle.subscription_id().into_uuid())
            })?;

        let charge = self
            .payment_gw
            .create_pending_charge(subscription.organization_id(), &cycle)
            .await?;

        // Stamp the gateway transaction id; outcome stays `Pending` until the
        // webhook subscriber resolves it via `RecordPaymentOutcomeUseCase`.
        let stamped = stamp_transaction(attempt, charge.transaction_id);
        self.dunning_repo.update(&stamped).await?;
        Ok(())
    }

    /// Transactional path — the attempt reads and the `transaction_id` stamp run
    /// inside the caller's tx so the stamp commits atomically with an
    /// audit-outbox event.
    ///
    /// Tradeoff: the payment-gateway call happens while the tx is open. With the
    /// v1.0 stub this is instant; a real (HTTP) gateway would hold the tx during
    /// the network round-trip. Acceptable here because the audited unit is the
    /// `transaction_id` stamp, not the charge itself (the charge is inherently
    /// non-transactional with the DB regardless).
    pub async fn execute_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        attempt_id: DunningAttemptId,
    ) -> Result<(), SubscriptionError> {
        let attempt = self
            .dunning_repo
            .find_by_id_in_tx(tx, attempt_id)
            .await?
            .ok_or_else(|| SubscriptionError::DunningAttemptNotFound(attempt_id.into_uuid()))?;

        // Already fired? Nothing to do — webhook will resolve it.
        if attempt.transaction_id().is_some() {
            return Ok(());
        }

        let cycle = self
            .cycle_repo
            .find_by_id_in_tx(tx, attempt.billing_cycle_id())
            .await?
            .ok_or_else(|| {
                SubscriptionError::BillingCycleNotFound(attempt.billing_cycle_id().into_uuid())
            })?;
        let subscription = self
            .sub_repo
            .find_by_id_in_tx(tx, cycle.subscription_id())
            .await?
            .ok_or_else(|| {
                SubscriptionError::SubscriptionNotFound(cycle.subscription_id().into_uuid())
            })?;

        let charge = self
            .payment_gw
            .create_pending_charge(subscription.organization_id(), &cycle)
            .await?;

        let stamped = stamp_transaction(attempt, charge.transaction_id);
        self.dunning_repo.update_in_tx(tx, &stamped).await?;
        Ok(())
    }
}

/// Stamps a `transaction_id` onto a `Pending` attempt by reconstituting it.
/// The entity intentionally does not expose a public setter — keeping this
/// helper local prevents accidental misuse from other modules.
fn stamp_transaction(attempt: DunningAttempt, transaction_id: Uuid) -> DunningAttempt {
    DunningAttempt::reconstitute(
        attempt.id(),
        attempt.billing_cycle_id(),
        attempt.attempt_number(),
        attempt.scheduled_at(),
        attempt.executed_at(),
        attempt.outcome(),
        attempt.failure_reason().map(str::to_string),
        Some(transaction_id),
        attempt.created_at(),
    )
}
