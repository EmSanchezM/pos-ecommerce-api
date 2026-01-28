// Purchasing routes for the API Gateway
//
// This module defines the purchasing routers that group all purchasing endpoints
// with authentication middleware.
//
// Vendors: /api/v1/vendors
// Purchase Orders: /api/v1/purchase-orders
// Goods Receipts: /api/v1/goods-receipts

use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

use crate::handlers::purchasing::{
    activate_vendor_handler, approve_purchase_order_handler, cancel_goods_receipt_handler,
    cancel_purchase_order_handler, close_purchase_order_handler, confirm_goods_receipt_handler,
    create_goods_receipt_handler, create_purchase_order_handler, create_vendor_handler,
    deactivate_vendor_handler, get_goods_receipt_handler, get_purchase_order_handler,
    get_vendor_handler, list_goods_receipts_handler, list_purchase_orders_handler,
    list_vendors_handler, reject_purchase_order_handler, submit_purchase_order_handler,
    update_vendor_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

/// Creates the vendors router with all vendor endpoints.
///
/// All routes require authentication via JWT token.
/// Operations require specific permissions.
///
/// # Routes
///
/// - `POST /` - Create a new vendor (requires vendors:create)
/// - `GET /` - List vendors with pagination and filters (requires vendors:read)
/// - `GET /{id}` - Get vendor details (requires vendors:read)
/// - `PUT /{id}` - Update vendor (requires vendors:update)
/// - `PUT /{id}/activate` - Activate vendor (requires vendors:update)
/// - `PUT /{id}/deactivate` - Deactivate vendor (requires vendors:update)
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/vendors", vendors_router(app_state.clone()))
///     .with_state(app_state);
/// ```
pub fn vendors_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Collection routes
        .route("/", post(create_vendor_handler).get(list_vendors_handler))
        // Individual vendor routes
        .route(
            "/{id}",
            get(get_vendor_handler).put(update_vendor_handler),
        )
        // Vendor status routes
        .route("/{id}/activate", put(activate_vendor_handler))
        .route("/{id}/deactivate", put(deactivate_vendor_handler))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the purchase orders router with all purchase order endpoints.
///
/// All routes require authentication via JWT token.
/// Operations require specific permissions.
///
/// # Routes
///
/// - `POST /` - Create a new purchase order (requires purchase_orders:create)
/// - `GET /` - List purchase orders with pagination and filters (requires purchase_orders:read)
/// - `GET /{id}` - Get purchase order details with items (requires purchase_orders:read)
/// - `PUT /{id}/submit` - Submit for approval (requires purchase_orders:submit)
/// - `PUT /{id}/approve` - Approve purchase order (requires purchase_orders:approve)
/// - `PUT /{id}/reject` - Reject purchase order (requires purchase_orders:approve)
/// - `PUT /{id}/cancel` - Cancel purchase order (requires purchase_orders:cancel)
/// - `PUT /{id}/close` - Close purchase order (requires purchase_orders:close)
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/purchase-orders", purchase_orders_router(app_state.clone()))
///     .with_state(app_state);
/// ```
pub fn purchase_orders_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Collection routes
        .route(
            "/",
            post(create_purchase_order_handler).get(list_purchase_orders_handler),
        )
        // Individual purchase order routes
        .route("/{id}", get(get_purchase_order_handler))
        // Purchase order workflow routes
        .route("/{id}/submit", put(submit_purchase_order_handler))
        .route("/{id}/approve", put(approve_purchase_order_handler))
        .route("/{id}/reject", put(reject_purchase_order_handler))
        .route("/{id}/cancel", put(cancel_purchase_order_handler))
        .route("/{id}/close", put(close_purchase_order_handler))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the goods receipts router with all goods receipt endpoints.
///
/// All routes require authentication via JWT token.
/// Operations require specific permissions.
///
/// # Routes
///
/// - `POST /` - Create a new goods receipt (requires goods_receipts:create)
/// - `GET /` - List goods receipts with pagination and filters (requires goods_receipts:read)
/// - `GET /{id}` - Get goods receipt details with items (requires goods_receipts:read)
/// - `PUT /{id}/confirm` - Confirm goods receipt (requires goods_receipts:confirm)
/// - `PUT /{id}/cancel` - Cancel goods receipt (requires goods_receipts:cancel)
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/goods-receipts", goods_receipts_router(app_state.clone()))
///     .with_state(app_state);
/// ```
pub fn goods_receipts_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Collection routes
        .route(
            "/",
            post(create_goods_receipt_handler).get(list_goods_receipts_handler),
        )
        // Individual goods receipt routes
        .route("/{id}", get(get_goods_receipt_handler))
        // Goods receipt workflow routes
        .route("/{id}/confirm", put(confirm_goods_receipt_handler))
        .route("/{id}/cancel", put(cancel_goods_receipt_handler))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
