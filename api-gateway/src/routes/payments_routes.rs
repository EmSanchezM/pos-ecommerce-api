// Payments routes for the API Gateway
//
// /api/v1/payment-gateways  - super-admin managed gateway catalog
// /api/v1/transactions      - charge/refund/list
// /api/v1/payouts           - read settlements
// /api/v1/webhooks          - public, signature-validated

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handlers::payments::{
    configure_gateway_handler, confirm_transaction_handler, delete_gateway_handler,
    get_gateway_handler, get_transaction_handler, handle_webhook_handler, list_gateways_handler,
    list_payouts_handler, list_transactions_handler, process_payment_handler,
    process_refund_handler, reconcile_transactions_handler, reject_transaction_handler,
    update_gateway_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

/// Payment gateway configuration. CUD endpoints are super-admin only at the
/// handler layer; the entire router is still behind the auth middleware so
/// reads can use a `payment_gateways:read` check.
pub fn payment_gateways_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(configure_gateway_handler).get(list_gateways_handler),
        )
        .route(
            "/{id}",
            get(get_gateway_handler)
                .put(update_gateway_handler)
                .delete(delete_gateway_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn transactions_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_transactions_handler))
        .route("/charge", post(process_payment_handler))
        .route("/reconcile", post(reconcile_transactions_handler))
        .route("/{id}", get(get_transaction_handler))
        .route("/{id}/refund", post(process_refund_handler))
        .route("/{id}/confirm", post(confirm_transaction_handler))
        .route("/{id}/reject", post(reject_transaction_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn payouts_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_payouts_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Webhooks DO NOT mount the auth middleware. Each gateway adapter is
/// responsible for validating the signature inside the use case.
pub fn webhooks_router() -> Router<AppState> {
    Router::new().route("/{gateway_type}", post(handle_webhook_handler))
}
