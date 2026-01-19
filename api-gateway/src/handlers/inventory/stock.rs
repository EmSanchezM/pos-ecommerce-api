// Stock HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for stock/inventory management:
// - GET /api/inventory/stock - List stock with pagination
// - GET /api/inventory/stock/{stock_id} - Get stock details
// - GET /api/stores/{store_id}/inventory - Get store inventory
// - GET /api/products/{product_id}/stock - Get product stock across stores

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use inventory::{
    GetProductStockUseCase, GetStockUseCase, GetStoreInventoryUseCase, ListStockQuery,
    ListStockUseCase, PaginatedResponse, StockDetailResponse, StockResponse,
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
) -> Result<Json<Vec<StockResponse>>, Response> {
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
