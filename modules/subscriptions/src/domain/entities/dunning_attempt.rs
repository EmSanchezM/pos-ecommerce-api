//! `DunningAttempt` — one retry against a failed `BillingCycle`.
//!
//! v1.0 dunning policy: 3 attempts at +1d, +3d, +7d after the initial
//! failure. After the third failure, the parent subscription transitions to
//! `PastDue` (and is canceled if not recovered within 14 grace days).

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::SubscriptionError;
use crate::domain::value_objects::{BillingCycleId, DunningAttemptId, DunningOutcome};

#[derive(Debug, Clone)]
pub struct DunningAttempt {
    id: DunningAttemptId,
    billing_cycle_id: BillingCycleId,
    /// 1-indexed retry number (1, 2, 3 in v1.0).
    attempt_number: i16,
    scheduled_at: DateTime<Utc>,
    executed_at: Option<DateTime<Utc>>,
    outcome: DunningOutcome,
    failure_reason: Option<String>,
    transaction_id: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl DunningAttempt {
    /// Creates a new attempt scheduled for `scheduled_at`. Outcome starts
    /// `Pending`; the cron job picks it up when `scheduled_at <= now()` and
    /// flips outcome via `mark_*`.
    pub fn schedule(
        billing_cycle_id: BillingCycleId,
        attempt_number: i16,
        scheduled_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: DunningAttemptId::new(),
            billing_cycle_id,
            attempt_number,
            scheduled_at,
            executed_at: None,
            outcome: DunningOutcome::Pending,
            failure_reason: None,
            transaction_id: None,
            created_at: Utc::now(),
        }
    }

    /// Rebuilds an attempt from its persisted form. Repository-only.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: DunningAttemptId,
        billing_cycle_id: BillingCycleId,
        attempt_number: i16,
        scheduled_at: DateTime<Utc>,
        executed_at: Option<DateTime<Utc>>,
        outcome: DunningOutcome,
        failure_reason: Option<String>,
        transaction_id: Option<Uuid>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            billing_cycle_id,
            attempt_number,
            scheduled_at,
            executed_at,
            outcome,
            failure_reason,
            transaction_id,
            created_at,
        }
    }

    fn check_pending(&self) -> Result<(), SubscriptionError> {
        if self.outcome != DunningOutcome::Pending {
            return Err(SubscriptionError::InvalidStatusTransition {
                from: self.outcome.as_str().to_string(),
                to: "<terminal>".to_string(),
            });
        }
        Ok(())
    }

    pub fn mark_succeeded(
        &mut self,
        transaction_id: Uuid,
        executed_at: DateTime<Utc>,
    ) -> Result<(), SubscriptionError> {
        self.check_pending()?;
        self.outcome = DunningOutcome::Succeeded;
        self.transaction_id = Some(transaction_id);
        self.executed_at = Some(executed_at);
        self.failure_reason = None;
        Ok(())
    }

    pub fn mark_failed(
        &mut self,
        reason: String,
        transaction_id: Uuid,
        executed_at: DateTime<Utc>,
    ) -> Result<(), SubscriptionError> {
        self.check_pending()?;
        self.outcome = DunningOutcome::Failed;
        self.transaction_id = Some(transaction_id);
        self.executed_at = Some(executed_at);
        self.failure_reason = Some(reason);
        Ok(())
    }

    pub fn mark_skipped(&mut self, executed_at: DateTime<Utc>) -> Result<(), SubscriptionError> {
        self.check_pending()?;
        self.outcome = DunningOutcome::Skipped;
        self.executed_at = Some(executed_at);
        Ok(())
    }

    // ---------------------------------------------------------------------
    // Getters
    // ---------------------------------------------------------------------

    pub fn id(&self) -> DunningAttemptId {
        self.id
    }
    pub fn billing_cycle_id(&self) -> BillingCycleId {
        self.billing_cycle_id
    }
    pub fn attempt_number(&self) -> i16 {
        self.attempt_number
    }
    pub fn scheduled_at(&self) -> DateTime<Utc> {
        self.scheduled_at
    }
    pub fn executed_at(&self) -> Option<DateTime<Utc>> {
        self.executed_at
    }
    pub fn outcome(&self) -> DunningOutcome {
        self.outcome
    }
    pub fn failure_reason(&self) -> Option<&str> {
        self.failure_reason.as_deref()
    }
    pub fn transaction_id(&self) -> Option<Uuid> {
        self.transaction_id
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
