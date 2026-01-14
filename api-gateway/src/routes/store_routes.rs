// Store routes for the API Gateway
//
// This module defines the store router that groups all store endpoints
// under /api/v1/stores prefix with authentication middleware.
//
// Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 4.1, 4.2

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::handlers::{
    activate_store_handler, create_store_handler, deactivate_store_handler, get_store_handler,
    list_stores_handler, update_store_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

/// Creates the store router with all store endpoints.
///
/// All routes require authentication via JWT token.
/// Some routes additionally require super_admin role.
///
/// # Routes
///
/// - `POST /` - Create a new store (requires super_admin) - Requirement 1.1, 1.5
/// - `GET /` - List stores with pagination and filters - Requirement 4.1
/// - `GET /:id` - Get store details with terminal count - Requirement 4.2
/// - `PUT /:id` - Update store details - Requirement 1.2
/// - `POST /:id/activate` - Activate store (requires super_admin) - Requirement 1.4, 1.5
/// - `POST /:id/deactivate` - Deactivate store and cascade to terminals (requires super_admin) - Requirement 1.3, 1.5
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/stores", store_router(app_state.clone()))
///     .with_state(app_state);
/// ```
pub fn store_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Collection routes
        .route("/", post(create_store_handler).get(list_stores_handler))
        // Individual store routes
        .route("/{id}", get(get_store_handler).put(update_store_handler))
        // Store activation routes
        .route("/{id}/activate", post(activate_store_handler))
        .route("/{id}/deactivate", post(deactivate_store_handler))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
