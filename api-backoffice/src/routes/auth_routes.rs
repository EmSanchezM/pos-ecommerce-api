use axum::{Router, routing::post};

use common::TrustedProxies;

use crate::handlers::auth::login_handler;
use crate::middleware::rate_limit::login_rate_limit_layer;
use crate::state::BackofficeAppState;

/// Router for public backoffice auth endpoints (no auth middleware applied).
///
/// Rate limited per client IP: this endpoint is unauthenticated by definition
/// and the credential behind it grants platform-owner access to every
/// organization.
pub fn auth_router(state: BackofficeAppState, trusted_proxies: TrustedProxies) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .layer(login_rate_limit_layer(trusted_proxies))
        .with_state(state)
}
