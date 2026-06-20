use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::plans::{
    create_plan_handler, deactivate_plan_handler, get_plan_handler, list_plans_handler,
    update_plan_handler,
};
use crate::state::BackofficeAppState;

/// Router for backoffice subscription-plan endpoints.
///
/// All routes require the backoffice auth middleware to have run; each handler
/// then enforces its own `platform:plan.*` permission.
pub fn plan_router(state: BackofficeAppState) -> Router {
    Router::new()
        .route("/", get(list_plans_handler).post(create_plan_handler))
        .route("/{id}", get(get_plan_handler).put(update_plan_handler))
        .route("/{id}/deactivate", post(deactivate_plan_handler))
        .with_state(state)
}
