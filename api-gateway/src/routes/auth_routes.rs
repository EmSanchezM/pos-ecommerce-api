// Authentication routes for the API Gateway
//
// This module defines the authentication router that groups all auth endpoints
// under /api/v1/auth prefix.

use axum::{Router, routing::post};

use crate::handlers::{login_handler, refresh_handler, register_handler};
use crate::middleware::rate_limit::auth_rate_limit_layer;
use crate::state::AppState;

/// Creates the authentication router with all auth endpoints.
///
/// Rate limiting is applied to login and register to prevent brute-force attacks.
///
/// # Routes
///
/// - `POST /register` - Register a new ecommerce user (Requirement 5.1)
/// - `POST /login` - Unified login with email or username (Requirement 5.3)
/// - `POST /refresh` - Refresh access token (Requirement 5.4)
pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/refresh", post(refresh_handler))
        .layer(auth_rate_limit_layer())
}
