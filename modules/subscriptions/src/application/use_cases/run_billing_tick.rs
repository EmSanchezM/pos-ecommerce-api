//! `run_billing_tick` — orchestrates the four steps the billing job performs
//! every hour: trial activation + period rollover, invoicing of pending
//! cycles, dunning retries, and grace-period cancellation.
//!
//! The function is callable from the api-gateway job loop *and* from
//! integration tests, which is why it lives here rather than in the
//! gateway crate.

use std::sync::Arc;

use chrono::Utc;

use crate::SubscriptionError;
use crate::application::dtos::BillingTickReport;
use crate::application::use_cases::{
    ProcessBillingCycleUseCase, ProcessDunningAttemptUseCase, TickPastDueSubscriptionsUseCase,
};
use crate::domain::entities::{BillingCycle, Subscription};
use crate::domain::repositories::{
    BillingCycleRepository, DunningAttemptRepository, SubscriptionPlanRepository,
    SubscriptionRepository,
};
use crate::domain::value_objects::{BillingCycleStatus, SubscriptionStatus};
use crate::infrastructure::{BillingInvoiceGateway, BillingPaymentGateway};

const DEFAULT_BATCH_SIZE: i64 = 100;

pub struct RunBillingTickUseCase {
    plan_repo: Arc<dyn SubscriptionPlanRepository>,
    sub_repo: Arc<dyn SubscriptionRepository>,
    cycle_repo: Arc<dyn BillingCycleRepository>,
    dunning_repo: Arc<dyn DunningAttemptRepository>,
    invoice_gw: Arc<dyn BillingInvoiceGateway>,
    payment_gw: Arc<dyn BillingPaymentGateway>,
}

impl RunBillingTickUseCase {
    pub fn new(
        plan_repo: Arc<dyn SubscriptionPlanRepository>,
        sub_repo: Arc<dyn SubscriptionRepository>,
        cycle_repo: Arc<dyn BillingCycleRepository>,
        dunning_repo: Arc<dyn DunningAttemptRepository>,
        invoice_gw: Arc<dyn BillingInvoiceGateway>,
        payment_gw: Arc<dyn BillingPaymentGateway>,
    ) -> Self {
        Self {
            plan_repo,
            sub_repo,
            cycle_repo,
            dunning_repo,
            invoice_gw,
            payment_gw,
        }
    }

    pub async fn execute(&self) -> Result<BillingTickReport, SubscriptionError> {
        let mut report = BillingTickReport::default();
        let now = Utc::now();

        // 1. Roll subscriptions whose period elapsed.
        let due = self
            .sub_repo
            .list_due_for_billing(now, DEFAULT_BATCH_SIZE)
            .await?;
        for sub in due {
            self.roll_subscription(sub, now, &mut report).await?;
        }

        // 2. Invoice + charge every Pending cycle whose period_start <= now.
        let process_cycle = ProcessBillingCycleUseCase::new(
            Arc::clone(&self.sub_repo),
            Arc::clone(&self.cycle_repo),
            Arc::clone(&self.invoice_gw),
            Arc::clone(&self.payment_gw),
        );
        let pending = self
            .cycle_repo
            .find_pending_due(now, DEFAULT_BATCH_SIZE)
            .await?;
        for cycle in pending {
            if let Err(e) = process_cycle.execute(cycle.id()).await {
                tracing::warn!(
                    cycle_id = %cycle.id().into_uuid(),
                    error = %e,
                    "process_billing_cycle failed"
                );
                continue;
            }
            report.cycles_invoiced += 1;
        }

        // 3. Fire dunning attempts whose `scheduled_at <= now`.
        let due_attempts = self.dunning_repo.find_due(now, DEFAULT_BATCH_SIZE).await?;
        let process_dunning = ProcessDunningAttemptUseCase::new(
            Arc::clone(&self.sub_repo),
            Arc::clone(&self.cycle_repo),
            Arc::clone(&self.dunning_repo),
            Arc::clone(&self.payment_gw),
        );
        for attempt in due_attempts {
            if let Err(e) = process_dunning.execute(attempt.id()).await {
                tracing::warn!(
                    attempt_id = %attempt.id().into_uuid(),
                    error = %e,
                    "process_dunning_attempt failed"
                );
                continue;
            }
            report.dunning_executed += 1;
        }

        // 4. Cancel past-due subs that exceeded the grace period.
        let past_due_uc = TickPastDueSubscriptionsUseCase::new(Arc::clone(&self.sub_repo));
        report.past_due_canceled = past_due_uc.execute().await?;

        Ok(report)
    }

    /// Either: trial expired → activate + open new Pending cycle. Or: regular
    /// period expired → advance period + open new Pending cycle. Either path
    /// uses `cancel_at_period_end` as a short-circuit to mark Canceled instead
    /// of rolling forward.
    async fn roll_subscription(
        &self,
        mut sub: Subscription,
        now: chrono::DateTime<Utc>,
        report: &mut BillingTickReport,
    ) -> Result<(), SubscriptionError> {
        let plan = self
            .plan_repo
            .find_by_id(sub.plan_id())
            .await?
            .ok_or_else(|| SubscriptionError::PlanNotFound(sub.plan_id().into_uuid()))?;

        // If the org asked to cancel at period end, terminate now and stop.
        if sub.cancel_at_period_end() {
            sub.cancel(false, now)?;
            self.sub_repo.update_with_version(&sub).await?;
            // TODO(events): publish `subscription.canceled`.
            return Ok(());
        }

        let was_trial = matches!(sub.status(), SubscriptionStatus::Trialing);
        if was_trial {
            sub.activate(plan.interval(), now)?;
            report.trial_activated += 1;
            // TODO(events): publish `subscription.activated`.
        } else {
            let next_end = plan.interval().next_period_end(sub.current_period_end());
            sub.advance_period(next_end);
            report.period_advanced += 1;
        }
        self.sub_repo.update_with_version(&sub).await?;

        // Open the next billing cycle (Pending — the cycle-processing step
        // below will invoice/charge it in the same tick if `period_start <= now`).
        let new_cycle = BillingCycle::create(
            sub.id(),
            sub.current_period_start(),
            sub.current_period_end(),
            plan.price_cents(),
            plan.currency().to_string(),
            BillingCycleStatus::Pending,
        );
        self.cycle_repo.save(&new_cycle).await?;
        Ok(())
    }
}
