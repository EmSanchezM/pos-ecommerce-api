//! `Subscription` — a single organization's recurring billing relationship
//! with the platform.
//!
//! Invariants:
//! - One non-`Canceled` row per `organization_id` (enforced by partial unique
//!   index in PostgreSQL; checked at the use-case layer too).
//! - Optimistic locking via `version`. Every mutating method bumps `version`
//!   and `updated_at`; the Pg repo enforces `WHERE version = expected - 1`
//!   on UPDATE, returning `OptimisticLockFailed` on a 0-row match.
//!
//! State machine (see `SubscriptionStatus::can_transition_to`):
//!
//! ```text
//!   Trialing ─► Active ─► PastDue ─► Active
//!      │          │          │          │
//!      └──────────┴──────────┴──────────┴─► Canceled
//! ```

use chrono::{DateTime, Duration, Utc};

use tenancy::OrganizationId;

use crate::SubscriptionError;
use crate::domain::entities::SubscriptionPlan;
use crate::domain::value_objects::{
    BillingInterval, SubscriptionId, SubscriptionPlanId, SubscriptionStatus,
};

#[derive(Debug, Clone)]
pub struct Subscription {
    id: SubscriptionId,
    organization_id: OrganizationId,
    plan_id: SubscriptionPlanId,
    status: SubscriptionStatus,
    current_period_start: DateTime<Utc>,
    current_period_end: DateTime<Utc>,
    trial_end: Option<DateTime<Utc>>,
    cancel_at_period_end: bool,
    canceled_at: Option<DateTime<Utc>>,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Subscription {
    /// Begins a new subscription for `organization_id` against `plan`.
    ///
    /// - If the plan defines `trial_days = Some(d)`, status starts as
    ///   `Trialing`, `trial_end = now + d`, and the first period mirrors the
    ///   trial window (so the cron job advances to `Active` exactly at trial
    ///   expiry).
    /// - Otherwise the subscription starts `Active` immediately and the first
    ///   period ends one cadence later.
    pub fn start(
        plan: &SubscriptionPlan,
        organization_id: OrganizationId,
        now: DateTime<Utc>,
    ) -> Self {
        let (status, trial_end, period_end) = match plan.trial_days() {
            Some(days) if days > 0 => {
                let end = now + Duration::days(days as i64);
                (SubscriptionStatus::Trialing, Some(end), end)
            }
            _ => (
                SubscriptionStatus::Active,
                None,
                plan.interval().next_period_end(now),
            ),
        };
        Self {
            id: SubscriptionId::new(),
            organization_id,
            plan_id: plan.id(),
            status,
            current_period_start: now,
            current_period_end: period_end,
            trial_end,
            cancel_at_period_end: false,
            canceled_at: None,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    /// Rebuilds a subscription from its persisted form. Repository-only.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: SubscriptionId,
        organization_id: OrganizationId,
        plan_id: SubscriptionPlanId,
        status: SubscriptionStatus,
        current_period_start: DateTime<Utc>,
        current_period_end: DateTime<Utc>,
        trial_end: Option<DateTime<Utc>>,
        cancel_at_period_end: bool,
        canceled_at: Option<DateTime<Utc>>,
        version: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            organization_id,
            plan_id,
            status,
            current_period_start,
            current_period_end,
            trial_end,
            cancel_at_period_end,
            canceled_at,
            version,
            created_at,
            updated_at,
        }
    }

    fn transition(&mut self, to: SubscriptionStatus) -> Result<(), SubscriptionError> {
        if !self.status.can_transition_to(&to) {
            return Err(SubscriptionError::InvalidStatusTransition {
                from: self.status.as_str().to_string(),
                to: to.as_str().to_string(),
            });
        }
        self.status = to;
        self.version += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Trial expired (or admin force-activates). Computes the new period end
    /// from `interval` so the entity stays decoupled from the plan repo.
    pub fn activate(
        &mut self,
        interval: BillingInterval,
        now: DateTime<Utc>,
    ) -> Result<(), SubscriptionError> {
        self.transition(SubscriptionStatus::Active)?;
        self.current_period_start = now;
        self.current_period_end = interval.next_period_end(now);
        Ok(())
    }

    /// Payment failed. Subscription is past_due; dunning attempts will retry.
    pub fn mark_past_due(&mut self) -> Result<(), SubscriptionError> {
        self.transition(SubscriptionStatus::PastDue)
    }

    /// Dunning succeeded — bring the subscription back to Active.
    pub fn resume_active(&mut self) -> Result<(), SubscriptionError> {
        self.transition(SubscriptionStatus::Active)
    }

    /// Cancel the subscription.
    ///
    /// - `at_period_end = true` (default in v1.0): defers cancellation —
    ///   leaves status untouched, sets `cancel_at_period_end = true`. The job
    ///   transitions to `Canceled` when `current_period_end <= now`.
    /// - `at_period_end = false`: immediately transitions to `Canceled`,
    ///   sets `canceled_at = now`. Reserved for super_admin.
    pub fn cancel(
        &mut self,
        at_period_end: bool,
        now: DateTime<Utc>,
    ) -> Result<(), SubscriptionError> {
        if at_period_end {
            if self.status == SubscriptionStatus::Canceled {
                return Err(SubscriptionError::InvalidStatusTransition {
                    from: self.status.as_str().to_string(),
                    to: SubscriptionStatus::Canceled.as_str().to_string(),
                });
            }
            self.cancel_at_period_end = true;
            self.version += 1;
            self.updated_at = now;
            return Ok(());
        }
        self.transition(SubscriptionStatus::Canceled)?;
        self.canceled_at = Some(now);
        Ok(())
    }

    /// Reverts a pending `cancel_at_period_end` while the subscription is
    /// still inside its paid period and not yet `Canceled`.
    pub fn resume(&mut self) -> Result<(), SubscriptionError> {
        if self.status == SubscriptionStatus::Canceled {
            return Err(SubscriptionError::InvalidStatusTransition {
                from: self.status.as_str().to_string(),
                to: "active".to_string(),
            });
        }
        if !self.cancel_at_period_end {
            return Err(SubscriptionError::Validation(
                "subscription is not pending cancellation".to_string(),
            ));
        }
        self.cancel_at_period_end = false;
        self.version += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Moves the period markers forward when the cron job opens a new
    /// billing cycle. `next_end` is computed by the caller (typically via
    /// `plan.interval().next_period_end(current_period_end)`).
    pub fn advance_period(&mut self, next_end: DateTime<Utc>) {
        self.current_period_start = self.current_period_end;
        self.current_period_end = next_end;
        self.version += 1;
        self.updated_at = Utc::now();
    }

    /// v1.0 plan-change semantics: just swap the `plan_id` and bump version.
    /// The job applies the effective change at the next period boundary
    /// (no proration in v1.0).
    pub fn change_plan(&mut self, new_plan_id: SubscriptionPlanId) {
        self.plan_id = new_plan_id;
        self.version += 1;
        self.updated_at = Utc::now();
    }

    // ---------------------------------------------------------------------
    // Getters
    // ---------------------------------------------------------------------

    pub fn id(&self) -> SubscriptionId {
        self.id
    }
    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
    }
    pub fn plan_id(&self) -> SubscriptionPlanId {
        self.plan_id
    }
    pub fn status(&self) -> SubscriptionStatus {
        self.status
    }
    pub fn current_period_start(&self) -> DateTime<Utc> {
        self.current_period_start
    }
    pub fn current_period_end(&self) -> DateTime<Utc> {
        self.current_period_end
    }
    pub fn trial_end(&self) -> Option<DateTime<Utc>> {
        self.trial_end
    }
    pub fn cancel_at_period_end(&self) -> bool {
        self.cancel_at_period_end
    }
    pub fn canceled_at(&self) -> Option<DateTime<Utc>> {
        self.canceled_at
    }
    pub fn version(&self) -> i32 {
        self.version
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::PlanCode;

    fn make_plan(trial_days: Option<i32>) -> SubscriptionPlan {
        let mut plan = SubscriptionPlan::create(
            PlanCode::new("pro_monthly").unwrap(),
            "Pro Monthly".to_string(),
            PlanTierShim::default_pro(),
            BillingInterval::Monthly,
            4900,
            "USD".to_string(),
        );
        plan.set_trial_days(trial_days);
        plan
    }

    // Tiny shim to avoid pulling tenancy::PlanTier directly into tests when
    // the real enum already lives in tenancy. Tests don't actually exercise
    // tier semantics, only the trial/period machinery.
    struct PlanTierShim;
    impl PlanTierShim {
        fn default_pro() -> tenancy::PlanTier {
            tenancy::PlanTier::Pro
        }
    }

    #[test]
    fn start_with_trial_sets_trialing_and_trial_end() {
        let plan = make_plan(Some(14));
        let now = Utc::now();
        let sub = Subscription::start(&plan, OrganizationId::new(), now);
        assert_eq!(sub.status(), SubscriptionStatus::Trialing);
        assert_eq!(sub.trial_end(), Some(now + Duration::days(14)));
        assert_eq!(sub.current_period_end(), now + Duration::days(14));
        assert_eq!(sub.version(), 1);
    }

    #[test]
    fn start_without_trial_is_active_immediately() {
        let plan = make_plan(None);
        let now = Utc::now();
        let sub = Subscription::start(&plan, OrganizationId::new(), now);
        assert_eq!(sub.status(), SubscriptionStatus::Active);
        assert!(sub.trial_end().is_none());
    }

    #[test]
    fn activate_from_trialing_bumps_version_and_period() {
        let plan = make_plan(Some(14));
        let now = Utc::now();
        let mut sub = Subscription::start(&plan, OrganizationId::new(), now);
        let activated_at = now + Duration::days(14);
        sub.activate(BillingInterval::Monthly, activated_at)
            .unwrap();
        assert_eq!(sub.status(), SubscriptionStatus::Active);
        assert_eq!(sub.version(), 2);
        assert_eq!(sub.current_period_start(), activated_at);
    }

    #[test]
    fn cancel_at_period_end_does_not_change_status() {
        let plan = make_plan(None);
        let now = Utc::now();
        let mut sub = Subscription::start(&plan, OrganizationId::new(), now);
        sub.cancel(true, now).unwrap();
        assert_eq!(sub.status(), SubscriptionStatus::Active);
        assert!(sub.cancel_at_period_end());
        assert_eq!(sub.version(), 2);
    }

    #[test]
    fn cancel_immediately_transitions_to_canceled() {
        let plan = make_plan(None);
        let now = Utc::now();
        let mut sub = Subscription::start(&plan, OrganizationId::new(), now);
        sub.cancel(false, now).unwrap();
        assert_eq!(sub.status(), SubscriptionStatus::Canceled);
        assert_eq!(sub.canceled_at(), Some(now));
    }

    #[test]
    fn resume_only_when_pending_cancel() {
        let plan = make_plan(None);
        let now = Utc::now();
        let mut sub = Subscription::start(&plan, OrganizationId::new(), now);
        assert!(sub.resume().is_err(), "no pending cancel");
        sub.cancel(true, now).unwrap();
        sub.resume().unwrap();
        assert!(!sub.cancel_at_period_end());
    }
}
