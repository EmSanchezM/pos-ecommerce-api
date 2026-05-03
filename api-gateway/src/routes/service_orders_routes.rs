// Service orders routes: assets, orders (with items/diagnostics/quotes/
// transitions), plus the PUBLIC status router (no auth, mirrors
// `public_booking_router`).
//
// /api/v1/assets                       - asset CRUD + history
// /api/v1/service-orders               - intake + list + detail + transitions
// /api/v1/public/service-orders/{id}   - PUBLIC status by token

use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::handlers::service_orders::{
    add_diagnostic_handler, add_item_handler, approve_quote_handler, cancel_service_order_handler,
    create_quote_handler, deactivate_asset_handler, deliver_service_order_handler,
    diagnose_service_order_handler, get_asset_handler, get_asset_history_handler,
    get_public_service_order_handler, get_service_order_handler, intake_service_order_handler,
    list_assets_handler, list_service_orders_handler, mark_ready_handler, register_asset_handler,
    reject_quote_handler, remove_item_handler, send_quote_handler, start_repair_handler,
    start_testing_handler, update_asset_handler, update_item_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn service_orders_assets_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_assets_handler).post(register_asset_handler))
        .route(
            "/{id}",
            get(get_asset_handler)
                .put(update_asset_handler)
                .delete(deactivate_asset_handler),
        )
        .route("/{id}/history", get(get_asset_history_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn service_orders_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_service_orders_handler).post(intake_service_order_handler),
        )
        .route("/{id}", get(get_service_order_handler))
        // Items
        .route("/{id}/items", post(add_item_handler))
        .route(
            "/{id}/items/{item_id}",
            put(update_item_handler).delete(remove_item_handler),
        )
        // Diagnostics
        .route("/{id}/diagnostics", post(add_diagnostic_handler))
        // Quotes
        .route("/{id}/quotes", post(create_quote_handler))
        .route("/{id}/quotes/{quote_id}/send", post(send_quote_handler))
        .route(
            "/{id}/quotes/{quote_id}/approve",
            post(approve_quote_handler),
        )
        .route("/{id}/quotes/{quote_id}/reject", post(reject_quote_handler))
        // Order workflow transitions
        .route("/{id}/diagnose", post(diagnose_service_order_handler))
        .route("/{id}/start-repair", post(start_repair_handler))
        .route("/{id}/start-testing", post(start_testing_handler))
        .route("/{id}/mark-ready", post(mark_ready_handler))
        .route("/{id}/deliver", post(deliver_service_order_handler))
        .route("/{id}/cancel", post(cancel_service_order_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// PUBLIC: no auth. Mounted under `/api/v1/public/service-orders`.
pub fn public_service_orders_router() -> Router<AppState> {
    Router::new().route("/{id}", get(get_public_service_order_handler))
}
