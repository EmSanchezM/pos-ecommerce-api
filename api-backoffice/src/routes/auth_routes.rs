use axum::{Router, routing::post};

use crate::handlers::auth::login_handler;
use crate::middleware::rate_limit::login_rate_limit_layer;
use crate::state::BackofficeAppState;

/// Router for public backoffice auth endpoints (no auth middleware applied).
///
/// Rate limited per peer IP: this endpoint is unauthenticated by definition and
/// the credential behind it grants platform-owner access to every organization.
pub fn auth_router(state: BackofficeAppState) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .layer(login_rate_limit_layer())
        .with_state(state)
}
