use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::subscriptions::{
    change_plan_handler, force_cancel_handler, get_subscription_handler, resume_handler,
};
use crate::state::BackofficeAppState;

/// Router for backoffice subscription-admin endpoints.
///
/// All routes require the backoffice auth middleware to have run; each handler
/// then enforces its own permission (`platform:org.list` for the read,
/// `platform:subscription.*` for the mutations).
pub fn subscription_router(state: BackofficeAppState) -> Router {
    Router::new()
        .route("/{org_id}", get(get_subscription_handler))
        .route("/{org_id}/force-cancel", post(force_cancel_handler))
        .route("/{org_id}/change-plan", post(change_plan_handler))
        .route("/{org_id}/resume", post(resume_handler))
        .with_state(state)
}
