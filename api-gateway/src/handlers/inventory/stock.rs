// Stock HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for stock/inventory management:
// - POST /api/inventory/stock - Initialize stock for a product in a store
// - POST /api/inventory/stock/bulk - Bulk initialize stock for multiple products
// - GET /api/inventory/stock - List stock with pagination
// - GET /api/inventory/stock/{stock_id} - Get stock details
// - PUT /api/inventory/stock/{stock_id}/levels - Update stock level thresholds
// - GET /api/stores/{store_id}/inventory - Get store inventory
// - GET /api/stores/{store_id}/low-stock - Get low stock alerts
// - GET /api/products/{product_id}/stock - Get product stock across stores

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use inventory::{
    BulkInitializeStockCommand, BulkInitializeStockResult, BulkInitializeStockUseCase,
    GetLowStockAlertsUseCase, GetProductStockUseCase, GetStockUseCase, GetStoreInventoryUseCase,
    InitializeStockCommand, InitializeStockUseCase, ListResponse, ListStockQuery, ListStockUseCase,
    PaginatedResponse, StockDetailResponse, StockResponse, UpdateStockLevelsCommand,
    UpdateStockLevelsUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

use super::products::{default_page, default_page_size};

// =============================================================================
// Query DTOs
// =============================================================================

/// Query parameters for listing stock (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct ListStockQueryParams {
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Filter by product ID
    pub product_id: Option<Uuid>,
    /// Filter to only show low stock items
    #[serde(default)]
    pub low_stock: bool,
}

impl From<ListStockQueryParams> for ListStockQuery {
    fn from(params: ListStockQueryParams) -> Self {
        ListStockQuery {
            store_id: params.store_id,
            product_id: params.product_id,
            low_stock: params.low_stock,
            page: params.page,
            page_size: params.page_size,
        }
    }
}

// =============================================================================
// List Stock Handler
// =============================================================================

/// Handler for GET /api/inventory/stock
///
/// Lists stock records with pagination and optional filters.
///
/// # Query Parameters
///
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `store_id` (optional): Filter by store
/// - `product_id` (optional): Filter by product
/// - `low_stock` (optional): Only show low stock items (default: false)
///
/// # Response
///
/// - 200 OK: Paginated list of stock records
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
pub async fn list_stock_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListStockQueryParams>,
) -> Result<Json<PaginatedResponse<StockResponse>>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = ListStockUseCase::new(state.stock_repo());

    let query: ListStockQuery = params.into();
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Stock Handler
// =============================================================================

/// Handler for GET /api/inventory/stock/{stock_id}
///
/// Gets detailed information about a specific stock record.
///
/// # Path Parameters
///
/// - `stock_id`: Stock record UUID
///
/// # Response
///
/// - 200 OK: Stock details with product/variant info
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
/// - 404 Not Found: Stock record doesn't exist
pub async fn get_stock_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(stock_id): Path<Uuid>,
) -> Result<Json<StockDetailResponse>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = GetStockUseCase::new(state.stock_repo(), state.product_repo());

    let response = use_case
        .execute(stock_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Store Inventory Handler
// =============================================================================

/// Handler for GET /api/stores/{store_id}/inventory
///
/// Gets all inventory for a specific store.
///
/// # Path Parameters
///
/// - `store_id`: Store UUID
///
/// # Response
///
/// - 200 OK: List of stock records for the store
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
pub async fn get_store_inventory_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(store_id): Path<Uuid>,
) -> Result<Json<ListResponse<StockResponse>>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = GetStoreInventoryUseCase::new(state.stock_repo());

    let response = use_case
        .execute(store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Product Stock Handler
// =============================================================================

/// Handler for GET /api/products/{product_id}/stock
///
/// Gets stock levels for a product across all stores.
///
/// # Path Parameters
///
/// - `product_id`: Product UUID
///
/// # Response
///
/// - 200 OK: List of stock records for the product across all stores
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
/// - 404 Not Found: Product doesn't exist
pub async fn get_product_stock_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<StockResponse>>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = GetProductStockUseCase::new(state.stock_repo(), state.product_repo());

    let response = use_case
        .execute(product_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Initialize Stock Handler
// =============================================================================

/// Handler for POST /api/inventory/stock
///
/// Initializes stock for a product or variant in a specific store.
/// Creates a new inventory_stock record.
///
/// # Request Body
///
/// ```json
/// {
///     "store_id": "uuid",
///     "product_id": "uuid",      // Either product_id OR variant_id
///     "variant_id": null,
///     "initial_quantity": 0,     // Optional, defaults to 0
///     "min_stock_level": 10,     // Optional, defaults to 0
///     "max_stock_level": 100     // Optional
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Stock record created
/// - 400 Bad Request: Validation error (must specify product_id OR variant_id)
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:write permission
/// - 404 Not Found: Product or variant doesn't exist
/// - 409 Conflict: Stock already exists for this store/product combination
pub async fn initialize_stock_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<InitializeStockCommand>,
) -> Result<(StatusCode, Json<StockResponse>), Response> {
    require_permission(&ctx, "inventory:write")?;

    let use_case = InitializeStockUseCase::new(
        state.stock_repo(),
        state.product_repo(),
        state.movement_repo(),
        state.audit_repo(),
    );

    let response = use_case
        .execute(command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

// =============================================================================
// Bulk Initialize Stock Handler
// =============================================================================

/// Handler for POST /api/inventory/stock/bulk
///
/// Initializes stock for multiple products/variants at once.
/// Useful for initial store setup or bulk imports.
///
/// # Request Body
///
/// ```json
/// {
///     "store_id": "uuid",
///     "items": [
///         {
///             "product_id": "uuid",
///             "initial_quantity": 50,
///             "min_stock_level": 10,
///             "max_stock_level": 200
///         },
///         {
///             "variant_id": "uuid",
///             "initial_quantity": 25,
///             "min_stock_level": 5
///         }
///     ]
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Returns results with successful and failed items
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:write permission
pub async fn bulk_initialize_stock_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<BulkInitializeStockCommand>,
) -> Result<Json<BulkInitializeStockResult>, Response> {
    require_permission(&ctx, "inventory:write")?;

    let use_case = BulkInitializeStockUseCase::new(
        state.stock_repo(),
        state.product_repo(),
        state.movement_repo(),
        state.audit_repo(),
    );

    let response = use_case
        .execute(command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Update Stock Levels Handler
// =============================================================================

/// Handler for PUT /api/inventory/stock/{stock_id}/levels
///
/// Updates the min/max stock level thresholds for a stock record.
/// Does not change the actual quantity.
///
/// # Path Parameters
///
/// - `stock_id`: Stock record UUID
///
/// # Request Body
///
/// ```json
/// {
///     "stock_id": "uuid",
///     "min_stock_level": 20,
///     "max_stock_level": 500,
///     "expected_version": 1
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Stock levels updated
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:write permission
/// - 404 Not Found: Stock record doesn't exist
/// - 409 Conflict: Version mismatch (optimistic locking)
pub async fn update_stock_levels_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(stock_id): Path<Uuid>,
    Json(mut command): Json<UpdateStockLevelsCommand>,
) -> Result<Json<StockResponse>, Response> {
    require_permission(&ctx, "inventory:write")?;

    // Ensure stock_id in path matches command
    command.stock_id = stock_id;

    let use_case = UpdateStockLevelsUseCase::new(state.stock_repo(), state.audit_repo());

    let response = use_case
        .execute(command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Low Stock Alerts Handler
// =============================================================================

/// Handler for GET /api/stores/{store_id}/low-stock
///
/// Gets all products with low stock levels for a specific store.
/// Useful for generating reorder alerts and notifications.
///
/// # Path Parameters
///
/// - `store_id`: Store UUID
///
/// # Response
///
/// - 200 OK: List of stock records with low stock
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
pub async fn get_low_stock_alerts_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(store_id): Path<Uuid>,
) -> Result<Json<ListResponse<StockResponse>>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = GetLowStockAlertsUseCase::new(state.stock_repo());

    let response = use_case
        .execute(store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
