//! Periodic billing job for the subscriptions module.
//!
//! Every `interval_secs` it runs `RunBillingTickUseCase`, which:
//!   1. activates trials whose `current_period_end` has elapsed,
//!   2. advances the period for active subs and opens the next `Pending`
//!      `BillingCycle`,
//!   3. invoices + charges every `Pending` cycle whose `period_start <= now`,
//!   4. fires due dunning attempts (stamping a fresh transaction id),
//!   5. cancels subscriptions stuck in `PastDue` past the 14-day grace period.
//!
//! The default cadence is hourly (matches the v1.0 plan in
//! `docs/roadmap-modulos.md`); the interval is configurable via
//! `SUBSCRIPTION_BILLING_INTERVAL_SECS`.

use std::sync::Arc;
use std::time::Duration;

use subscriptions::{
    BillingCycleRepository, BillingInvoiceGateway, BillingPaymentGateway, DunningAttemptRepository,
    RunBillingTickUseCase, SubscriptionPlanRepository, SubscriptionRepository,
};

#[allow(clippy::too_many_arguments)]
pub fn spawn(
    plan_repo: Arc<dyn SubscriptionPlanRepository>,
    sub_repo: Arc<dyn SubscriptionRepository>,
    cycle_repo: Arc<dyn BillingCycleRepository>,
    dunning_repo: Arc<dyn DunningAttemptRepository>,
    invoice_gw: Arc<dyn BillingInvoiceGateway>,
    payment_gw: Arc<dyn BillingPaymentGateway>,
    interval_secs: u64,
) {
    let use_case = RunBillingTickUseCase::new(
        plan_repo,
        sub_repo,
        cycle_repo,
        dunning_repo,
        invoice_gw,
        payment_gw,
    );
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        // Skip the first tick — `tokio::time::interval` fires immediately on
        // creation, and we want to give the rest of the boot a beat to settle.
        interval.tick().await;

        loop {
            interval.tick().await;
            match use_case.execute().await {
                Ok(report) => {
                    if report.trial_activated > 0
                        || report.period_advanced > 0
                        || report.cycles_invoiced > 0
                        || report.dunning_executed > 0
                        || report.past_due_canceled > 0
                    {
                        tracing::info!(
                            trial_activated = report.trial_activated,
                            period_advanced = report.period_advanced,
                            cycles_invoiced = report.cycles_invoiced,
                            dunning_executed = report.dunning_executed,
                            past_due_canceled = report.past_due_canceled,
                            "[subscription-billing] tick complete"
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "[subscription-billing] tick failed");
                }
            }
        }
    });
}
