// Authentication routes for the API Gateway
//
// This module defines the authentication router that groups all auth endpoints
// under /api/v1/auth prefix.
//
// Requirements: 5.1, 5.3, 5.4

use axum::{routing::post, Router};

use crate::handlers::{login_handler, refresh_handler, register_handler};
use crate::state::AppState;

/// Creates the authentication router with all auth endpoints.
///
/// # Routes
///
/// - `POST /register` - Register a new ecommerce user (Requirement 5.1)
/// - `POST /login` - Unified login with email or username (Requirement 5.3)
/// - `POST /refresh` - Refresh access token (Requirement 5.4)
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/auth", auth_router())
///     .with_state(app_state);
/// ```
pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/refresh", post(refresh_handler))
}
