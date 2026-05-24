// Impersonation routes
//
// POST /backoffice/impersonate/{tenant_user_id}
// Permission: platform:user.impersonate (enforced in the handler)

use axum::{Router, routing::post};

use crate::handlers::impersonate::impersonate_handler;
use crate::state::BackofficeAppState;

/// Router for the impersonation endpoint.
///
/// All routes here require the backoffice auth middleware to have run.
pub fn impersonate_router(state: BackofficeAppState) -> Router {
    Router::new()
        .route("/{tenant_user_id}", post(impersonate_handler))
        .with_state(state)
}
