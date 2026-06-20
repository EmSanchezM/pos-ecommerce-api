use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::orgs::{list_orgs_handler, suspend_org_handler};
use crate::state::BackofficeAppState;

/// Router for backoffice organization endpoints.
///
/// All routes here require the backoffice auth middleware to have run.
pub fn org_router(state: BackofficeAppState) -> Router {
    Router::new()
        .route("/", get(list_orgs_handler))
        .route("/{id}/suspend", post(suspend_org_handler))
        .with_state(state)
}
