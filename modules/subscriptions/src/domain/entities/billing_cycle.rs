//! `BillingCycle` — one row per period charged to a `Subscription`.
//!
//! Lifecycle (see `BillingCycleStatus`):
//!
//! - `Pending`  — created by the cron job at the period boundary; awaiting
//!   invoice + transaction.
//! - `Trialing` — first cycle for trials with `trial_days > 0`; never
//!   produces an invoice. Auto-Skipped when the trial expires.
//! - `Invoiced` — invoice generated via `fiscal::invoice`, `Transaction`
//!   created via `payments` (status pending). Awaits payment confirmation.
//! - `Paid`     — gateway webhook confirmed payment.
//! - `Failed`   — gateway webhook reported failure; dunning attempts kick in.
//! - `Skipped`  — administrative skip (or trial cycle).

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::SubscriptionError;
use crate::domain::value_objects::{BillingCycleId, BillingCycleStatus, SubscriptionId};

#[derive(Debug, Clone)]
pub struct BillingCycle {
    id: BillingCycleId,
    subscription_id: SubscriptionId,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    status: BillingCycleStatus,
    invoice_id: Option<Uuid>,
    transaction_id: Option<Uuid>,
    /// Period amount in the smallest currency unit (cents).
    amount_cents: i64,
    /// ISO-4217 currency code.
    currency: String,
    attempted_at: Option<DateTime<Utc>>,
    settled_at: Option<DateTime<Utc>>,
    failure_reason: Option<String>,
    created_at: DateTime<Utc>,
}

impl BillingCycle {
    pub fn create(
        subscription_id: SubscriptionId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        amount_cents: i64,
        currency: String,
        status: BillingCycleStatus,
    ) -> Self {
        Self {
            id: BillingCycleId::new(),
            subscription_id,
            period_start,
            period_end,
            status,
            invoice_id: None,
            transaction_id: None,
            amount_cents,
            currency,
            attempted_at: None,
            settled_at: None,
            failure_reason: None,
            created_at: Utc::now(),
        }
    }

    /// Rebuilds a cycle from its persisted form. Repository-only.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: BillingCycleId,
        subscription_id: SubscriptionId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        status: BillingCycleStatus,
        invoice_id: Option<Uuid>,
        transaction_id: Option<Uuid>,
        amount_cents: i64,
        currency: String,
        attempted_at: Option<DateTime<Utc>>,
        settled_at: Option<DateTime<Utc>>,
        failure_reason: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            subscription_id,
            period_start,
            period_end,
            status,
            invoice_id,
            transaction_id,
            amount_cents,
            currency,
            attempted_at,
            settled_at,
            failure_reason,
            created_at,
        }
    }

    fn check_transition(&self, to: &BillingCycleStatus) -> Result<(), SubscriptionError> {
        if !self.status.can_transition_to(to) {
            return Err(SubscriptionError::InvalidStatusTransition {
                from: self.status.as_str().to_string(),
                to: to.as_str().to_string(),
            });
        }
        Ok(())
    }

    /// `Pending → Invoiced`. The job has generated an invoice + a pending
    /// `Transaction`; both ids are stamped on the cycle.
    pub fn mark_invoiced(
        &mut self,
        invoice_id: Uuid,
        transaction_id: Uuid,
        attempted_at: DateTime<Utc>,
    ) -> Result<(), SubscriptionError> {
        self.check_transition(&BillingCycleStatus::Invoiced)?;
        self.status = BillingCycleStatus::Invoiced;
        self.invoice_id = Some(invoice_id);
        self.transaction_id = Some(transaction_id);
        self.attempted_at = Some(attempted_at);
        Ok(())
    }

    /// `Invoiced → Paid` (or `Failed → Paid` after a successful dunning).
    pub fn mark_paid(&mut self, settled_at: DateTime<Utc>) -> Result<(), SubscriptionError> {
        self.check_transition(&BillingCycleStatus::Paid)?;
        self.status = BillingCycleStatus::Paid;
        self.settled_at = Some(settled_at);
        self.failure_reason = None;
        Ok(())
    }

    /// `Invoiced → Failed`. Dunning will be scheduled by the application
    /// layer after this transition.
    pub fn mark_failed(
        &mut self,
        reason: String,
        attempted_at: DateTime<Utc>,
    ) -> Result<(), SubscriptionError> {
        self.check_transition(&BillingCycleStatus::Failed)?;
        self.status = BillingCycleStatus::Failed;
        self.failure_reason = Some(reason);
        self.attempted_at = Some(attempted_at);
        Ok(())
    }

    /// `Pending|Trialing → Skipped`. Used for trial cycles (no invoice ever
    /// generated) or admin overrides.
    pub fn mark_skipped(&mut self, now: DateTime<Utc>) -> Result<(), SubscriptionError> {
        self.check_transition(&BillingCycleStatus::Skipped)?;
        self.status = BillingCycleStatus::Skipped;
        self.settled_at = Some(now);
        Ok(())
    }

    // ---------------------------------------------------------------------
    // Getters
    // ---------------------------------------------------------------------

    pub fn id(&self) -> BillingCycleId {
        self.id
    }
    pub fn subscription_id(&self) -> SubscriptionId {
        self.subscription_id
    }
    pub fn period_start(&self) -> DateTime<Utc> {
        self.period_start
    }
    pub fn period_end(&self) -> DateTime<Utc> {
        self.period_end
    }
    pub fn status(&self) -> BillingCycleStatus {
        self.status
    }
    pub fn invoice_id(&self) -> Option<Uuid> {
        self.invoice_id
    }
    pub fn transaction_id(&self) -> Option<Uuid> {
        self.transaction_id
    }
    pub fn amount_cents(&self) -> i64 {
        self.amount_cents
    }
    pub fn currency(&self) -> &str {
        &self.currency
    }
    pub fn attempted_at(&self) -> Option<DateTime<Utc>> {
        self.attempted_at
    }
    pub fn settled_at(&self) -> Option<DateTime<Utc>> {
        self.settled_at
    }
    pub fn failure_reason(&self) -> Option<&str> {
        self.failure_reason.as_deref()
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
