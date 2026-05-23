pub mod analytics_recompute;
pub mod cart_cleanup;
pub mod demand_planning_recompute;
pub mod event_dispatcher;
pub mod notification_dispatcher;
pub mod reservation_expiry;
pub mod subscription_billing;

use crate::config::JobsConfig;
use crate::state::AppState;

pub fn spawn_all(state: &AppState, config: &JobsConfig) {
    reservation_expiry::spawn(
        state.reservation_repo(),
        state.stock_repo(),
        config.reservation_expiry_interval,
    );
    cart_cleanup::spawn(state.cart_repo(), config.cart_cleanup_interval);
    event_dispatcher::spawn(
        state.outbox_repo(),
        state.subscriber_registry(),
        config.event_dispatch_interval,
        config.event_dispatch_batch_size,
    );
    notification_dispatcher::spawn(
        state.notification_repo(),
        state.notification_registry(),
        config.notification_retry_interval,
        config.notification_retry_batch_size,
    );
    analytics_recompute::spawn(
        state.analytics_query_repo(),
        state.kpi_snapshot_repo(),
        config.analytics_recompute_interval,
    );
    demand_planning_recompute::spawn(
        state.sales_history_repo(),
        state.demand_forecast_repo(),
        state.reorder_policy_repo(),
        state.stock_snapshot_repo(),
        state.replenishment_suggestion_repo(),
        state.abc_classification_repo(),
        config.demand_planning_interval,
    );
    subscription_billing::spawn(
        state.subscription_plan_repo(),
        state.subscription_repo(),
        state.billing_cycle_repo(),
        state.dunning_attempt_repo(),
        state.subscription_invoice_gateway(),
        state.subscription_payment_gateway(),
        config.subscription_billing_interval,
    );
}
