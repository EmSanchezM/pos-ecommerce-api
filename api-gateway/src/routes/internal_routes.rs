// Internal service-to-service routes (no tenant auth middleware).
//
// Mounted under /internal. Authenticated by a shared internal secret checked
// inside the handler, not by the tenant JWT middleware — these are
// service-to-service calls, not tenant requests.

use axum::{Router, routing::post};

use crate::handlers::internal::{InternalState, issue_impersonation_token_handler};

/// Router for `/internal/*`. Carries its own `InternalState` so it does not
/// depend on the tenant `AppState` / auth middleware.
pub fn internal_router(state: InternalState) -> Router {
    Router::new()
        .route(
            "/issue-impersonation-token",
            post(issue_impersonation_token_handler),
        )
        .with_state(state)
}
