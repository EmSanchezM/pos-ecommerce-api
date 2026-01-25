// Inventory routes for the API Gateway
//
// This module defines the inventory routers that group all inventory endpoints
// with authentication middleware.
//
// Products: /api/v1/products
// Recipes: /api/v1/recipes
// Inventory: /api/v1/inventory

use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

use crate::handlers::{
    apply_adjustment_handler, approve_adjustment_handler, bulk_initialize_stock_handler,
    calculate_recipe_cost_handler, cancel_reservation_handler, confirm_reservation_handler,
    create_adjustment_handler, create_product_handler, create_recipe_handler,
    create_reservation_handler, create_variant_handler, delete_product_handler,
    delete_variant_handler, expire_reservations_handler, get_adjustment_handler,
    get_low_stock_report_handler, get_movements_report_handler, get_product_handler,
    get_product_recipe_handler, get_product_stock_handler, get_recipe_handler,
    get_stock_handler, get_stock_history_handler, get_valuation_report_handler,
    get_variant_handler, initialize_stock_handler, list_adjustments_handler,
    list_products_handler, list_recipes_handler, list_reservations_handler, list_stock_handler,
    list_variants_handler, reject_adjustment_handler, submit_adjustment_handler,
    update_product_handler, update_recipe_handler, update_stock_levels_handler,
    update_variant_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

/// Creates the products router with all product endpoints.
///
/// All routes require authentication via JWT token.
/// Create, update, and delete operations require specific permissions.
///
/// # Routes
///
/// - `POST /` - Create a new product (requires products:create)
/// - `GET /` - List products with pagination and filters
/// - `GET /{id}` - Get product details with variants
/// - `PUT /{id}` - Update product (requires products:update)
/// - `DELETE /{id}` - Soft delete product (requires products:delete)
/// - `POST /{product_id}/variants` - Create variant (requires products:create)
/// - `GET /{product_id}/variants` - List variants
/// - `GET /{product_id}/variants/{variant_id}` - Get variant details
/// - `PUT /{product_id}/variants/{variant_id}` - Update variant (requires products:update)
/// - `DELETE /{product_id}/variants/{variant_id}` - Delete variant (requires products:delete)
/// - `GET /{product_id}/recipe` - Get active recipe for product
/// - `GET /{product_id}/stock` - Get product stock across all stores
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/products", products_router(app_state.clone()))
///     .with_state(app_state);
/// ```
pub fn products_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Collection routes
        .route("/", post(create_product_handler).get(list_products_handler))
        // Individual product routes
        .route(
            "/{id}",
            get(get_product_handler)
                .put(update_product_handler)
                .delete(delete_product_handler),
        )
        // Variant collection routes
        .route(
            "/{product_id}/variants",
            post(create_variant_handler).get(list_variants_handler),
        )
        // Individual variant routes
        .route(
            "/{product_id}/variants/{variant_id}",
            get(get_variant_handler)
                .put(update_variant_handler)
                .delete(delete_variant_handler),
        )
        // Product recipe route
        .route("/{product_id}/recipe", get(get_product_recipe_handler))
        // Product stock route
        .route("/{product_id}/stock", get(get_product_stock_handler))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the recipes router with all recipe endpoints.
///
/// All routes require authentication via JWT token.
/// Create and update operations require specific permissions.
///
/// # Routes
///
/// - `POST /` - Create a new recipe (requires recipes:create)
/// - `GET /` - List recipes with pagination and filters
/// - `GET /{id}` - Get recipe details with ingredients
/// - `PUT /{id}` - Update recipe (requires recipes:update)
/// - `POST /{recipe_id}/calculate-cost` - Calculate recipe cost
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/recipes", recipes_router(app_state.clone()))
///     .with_state(app_state);
/// ```
pub fn recipes_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Collection routes
        .route("/", post(create_recipe_handler).get(list_recipes_handler))
        // Individual recipe routes
        .route(
            "/{id}",
            get(get_recipe_handler).put(update_recipe_handler),
        )
        // Recipe cost calculation
        .route("/{recipe_id}/calculate-cost", post(calculate_recipe_cost_handler))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the inventory router with stock, reservation, and adjustment endpoints.
///
/// All routes require authentication via JWT token.
/// Stock operations require inventory:read permission.
/// Reservation operations require various permissions based on the action.
/// Adjustment operations require inventory:adjustments:* permissions.
///
/// # Routes
///
/// ## Stock Routes
/// - `POST /stock` - Initialize stock for a product in a store (requires inventory:write)
/// - `POST /stock/bulk` - Bulk initialize stock for multiple products (requires inventory:write)
/// - `GET /stock` - List stock with pagination and filters
/// - `GET /stock/{stock_id}` - Get stock details
/// - `PUT /stock/{stock_id}/levels` - Update stock level thresholds (requires inventory:write)
/// - `GET /stock/{stock_id}/history` - Get stock movement history (requires inventory:read)
///
/// ## Reservation Routes
/// - `POST /reservations` - Create a reservation (requires cart:add or sales:create)
/// - `GET /reservations` - List reservations (requires inventory:read)
/// - `PUT /reservations/{id}/confirm` - Confirm a reservation (requires sales:create)
/// - `PUT /reservations/{id}/cancel` - Cancel a reservation (requires cart:remove or sales:void)
/// - `POST /reservations/expire` - Expire all expired reservations (requires system:admin)
///
/// ## Adjustment Routes
/// - `POST /adjustments` - Create an adjustment (requires inventory:adjustments:create)
/// - `GET /adjustments` - List adjustments (requires inventory:adjustments:read)
/// - `GET /adjustments/{id}` - Get adjustment details (requires inventory:adjustments:read)
/// - `PUT /adjustments/{id}/submit` - Submit for approval (requires inventory:adjustments:submit)
/// - `PUT /adjustments/{id}/approve` - Approve adjustment (requires inventory:adjustments:approve)
/// - `PUT /adjustments/{id}/reject` - Reject adjustment (requires inventory:adjustments:approve)
/// - `POST /adjustments/{id}/apply` - Apply to stock (requires inventory:adjustments:apply)
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/inventory", inventory_router(app_state.clone()))
///     .with_state(app_state);
/// ```
pub fn inventory_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Stock collection routes
        .route("/stock", post(initialize_stock_handler).get(list_stock_handler))
        // Bulk stock initialization
        .route("/stock/bulk", post(bulk_initialize_stock_handler))
        // Individual stock routes
        .route("/stock/{stock_id}", get(get_stock_handler))
        // Stock levels update
        .route("/stock/{stock_id}/levels", put(update_stock_levels_handler))
        // Stock history route
        .route("/stock/{stock_id}/history", get(get_stock_history_handler))
        // Reservation collection routes
        .route(
            "/reservations",
            post(create_reservation_handler).get(list_reservations_handler),
        )
        // Reservation action routes
        .route("/reservations/{id}/confirm", put(confirm_reservation_handler))
        .route("/reservations/{id}/cancel", put(cancel_reservation_handler))
        // Reservation batch operations
        .route("/reservations/expire", post(expire_reservations_handler))
        // Adjustment collection routes
        .route(
            "/adjustments",
            post(create_adjustment_handler).get(list_adjustments_handler),
        )
        // Individual adjustment routes
        .route("/adjustments/{id}", get(get_adjustment_handler))
        // Adjustment workflow routes
        .route("/adjustments/{id}/submit", put(submit_adjustment_handler))
        .route("/adjustments/{id}/approve", put(approve_adjustment_handler))
        .route("/adjustments/{id}/reject", put(reject_adjustment_handler))
        .route("/adjustments/{id}/apply", post(apply_adjustment_handler))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the inventory reports router with all report endpoints.
///
/// All routes require authentication via JWT token.
/// Report operations require reports:inventory permission.
///
/// # Routes
///
/// - `GET /inventory/valuation` - Get inventory valuation report (requires reports:inventory)
/// - `GET /inventory/low-stock` - Get low stock report (requires reports:inventory)
/// - `GET /inventory/movements` - Get movements report (requires reports:inventory)
///
/// # Usage
///
/// ```rust,ignore
/// let app = Router::new()
///     .nest("/api/v1/reports", reports_router(app_state.clone()))
///     .with_state(app_state);
/// ```
pub fn reports_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Inventory reports
        .route("/inventory/valuation", get(get_valuation_report_handler))
        .route("/inventory/low-stock", get(get_low_stock_report_handler))
        .route("/inventory/movements", get(get_movements_report_handler))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
