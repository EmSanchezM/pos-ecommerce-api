// Terminal routes for the API Gateway
//
// This module defines the terminal router that groups all terminal endpoints
// with authentication middleware.

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::handlers::{
    activate_terminal_handler, assign_cai_handler, create_terminal_handler,
    deactivate_terminal_handler, get_cai_history_handler, get_cai_status_handler,
    get_next_invoice_number_handler, get_terminal_handler, list_terminals_handler,
    update_terminal_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

/// Creates the terminal router for store-scoped terminal operations.
///
/// These routes are nested under /api/v1/stores/:store_id/terminals
/// All routes require authentication via JWT token.
///
/// # Routes
///
/// - `POST /` - Create a new terminal (requires super_admin) - Requirement 2.1, 2.4
/// - `GET /` - List terminals for a store with CAI status - Requirement 4.3
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/stores/:store_id/terminals", store_terminals_router(app_state.clone()))
///     .with_state(app_state);
/// ```
pub fn store_terminals_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_terminal_handler).get(list_terminals_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the terminal router for direct terminal operations.
///
/// These routes are at /api/v1/terminals/:id
/// All routes require authentication via JWT token.
///
/// # Routes
///
/// - `GET /:id` - Get terminal details with CAI history - Requirement 4.4
/// - `PUT /:id` - Update terminal details
/// - `POST /:id/activate` - Activate terminal - Requirement 2.6
/// - `POST /:id/deactivate` - Deactivate terminal preserving CAI history - Requirement 2.6
/// - `POST /:id/cai` - Assign CAI to terminal (requires super_admin) - Requirement 2.2, 2.3
/// - `GET /:id/cai/status` - Get CAI status with expiration warning - Requirement 3.4, 3.5
/// - `POST /:id/cai/next-number` - Get next invoice number atomically - Requirement 3.1
/// - `GET /:id/cai/history` - Get complete CAI history - Requirement 4.4
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/terminals", terminals_router(app_state.clone()))
///     .with_state(app_state);
/// ```
pub fn terminals_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Individual terminal routes
        .route("/{id}", get(get_terminal_handler).put(update_terminal_handler))
        // Terminal activation routes
        .route("/{id}/activate", post(activate_terminal_handler))
        .route("/{id}/deactivate", post(deactivate_terminal_handler))
        // CAI management routes
        .route("/{id}/cai", post(assign_cai_handler))
        .route("/{id}/cai/status", get(get_cai_status_handler))
        .route("/{id}/cai/next-number", post(get_next_invoice_number_handler))
        .route("/{id}/cai/history", get(get_cai_history_handler))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
