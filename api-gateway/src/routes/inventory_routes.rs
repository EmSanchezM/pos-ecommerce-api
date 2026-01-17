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
    routing::{get, post},
    Router,
};

use crate::handlers::{
    calculate_recipe_cost_handler, create_product_handler, create_recipe_handler,
    create_variant_handler, delete_product_handler, delete_variant_handler, get_product_handler,
    get_product_recipe_handler, get_product_stock_handler, get_recipe_handler, get_stock_handler,
    get_variant_handler, list_products_handler, list_recipes_handler, list_stock_handler,
    list_variants_handler, update_product_handler, update_recipe_handler, update_variant_handler,
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

/// Creates the inventory router with stock query endpoints.
///
/// All routes require authentication via JWT token.
/// All operations require inventory:read permission.
///
/// # Routes
///
/// - `GET /stock` - List stock with pagination and filters
/// - `GET /stock/{stock_id}` - Get stock details
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
        .route("/stock", get(list_stock_handler))
        // Individual stock routes
        .route("/stock/{stock_id}", get(get_stock_handler))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
