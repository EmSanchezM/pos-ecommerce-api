// Demand planning routes: forecasts, reorder policies, replenishment
// suggestions, ABC classification.

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handlers::demand_planning::{
    approve_suggestion_handler, dismiss_suggestion_handler, get_forecast_handler, list_abc_handler,
    list_reorder_policies_handler, list_replenishment_suggestions_handler,
    upsert_reorder_policy_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn forecasts_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/products/{variant_id}", get(get_forecast_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn reorder_policies_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_reorder_policies_handler).post(upsert_reorder_policy_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn replenishment_suggestions_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_replenishment_suggestions_handler))
        .route("/{id}/approve", post(approve_suggestion_handler))
        .route("/{id}/dismiss", post(dismiss_suggestion_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn abc_classification_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_abc_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
