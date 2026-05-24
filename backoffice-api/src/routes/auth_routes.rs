use axum::{Router, routing::post};

use crate::handlers::auth::login_handler;
use crate::state::BackofficeAppState;

/// Router for public backoffice auth endpoints (no auth middleware applied).
pub fn auth_router(state: BackofficeAppState) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .with_state(state)
}
