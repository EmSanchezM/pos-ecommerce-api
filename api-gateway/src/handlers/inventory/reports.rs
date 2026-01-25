// Stock History and Report HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for stock history and inventory reports:
// - GET /api/inventory/stock/{stock_id}/history - Get stock history (movements)
// - GET /api/products/{product_id}/stock-history - Get product stock history across stores
// - GET /api/reports/inventory/valuation - Get inventory valuation report
// - GET /api/reports/inventory/low-stock - Get low stock report
// - GET /api/reports/inventory/movements - Get movements report

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use inventory::{
    GetLowStockReportUseCase, GetMovementsReportUseCase, GetStockHistoryUseCase,
    GetValuationReportUseCase, LowStockReportQuery, LowStockReportResponse,
    MovementsReportQuery, MovementResponse, PaginatedResponse, StockHistoryQuery,
    StockHistoryResponse, ValuationReportQuery, ValuationReportResponse,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

use super::products::{default_page, default_page_size};

// =============================================================================
// Query DTOs
// =============================================================================

/// Query parameters for stock history (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct StockHistoryQueryParams {
    /// Filter movements from this date (inclusive)
    pub from_date: Option<DateTime<Utc>>,
    /// Filter movements to this date (inclusive)
    pub to_date: Option<DateTime<Utc>>,
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

/// Query parameters for valuation report (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct ValuationReportQueryParams {
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Currency for the report (defaults to HNL)
    pub currency: Option<String>,
}

/// Query parameters for low stock report (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct LowStockReportQueryParams {
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Include items with zero stock (default: true)
    #[serde(default = "default_include_zero")]
    pub include_zero_stock: bool,
}

fn default_include_zero() -> bool {
    true
}

/// Query parameters for movements report (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct MovementsReportQueryParams {
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Filter by stock ID
    pub stock_id: Option<Uuid>,
    /// Filter by movement type (e.g., "in", "out", "adjustment")
    pub movement_type: Option<String>,
    /// Filter movements from this date (inclusive)
    pub from_date: Option<DateTime<Utc>>,
    /// Filter movements to this date (inclusive)
    pub to_date: Option<DateTime<Utc>>,
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

// =============================================================================
// Get Stock History Handler
// =============================================================================

/// Handler for GET /api/inventory/stock/{stock_id}/history
///
/// Gets the movement history for a specific stock record.
///
/// # Path Parameters
///
/// - `stock_id`: Stock record UUID
///
/// # Query Parameters
///
/// - `from_date` (optional): Filter movements from this date
/// - `to_date` (optional): Filter movements to this date
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
///
/// # Response
///
/// - 200 OK: Stock history with movements
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
/// - 404 Not Found: Stock record doesn't exist
pub async fn get_stock_history_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(stock_id): Path<Uuid>,
    Query(params): Query<StockHistoryQueryParams>,
) -> Result<Json<StockHistoryResponse>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = GetStockHistoryUseCase::new(
        state.stock_repo(),
        state.movement_repo(),
        state.product_repo(),
    );

    let query = StockHistoryQuery {
        stock_id,
        from_date: params.from_date,
        to_date: params.to_date,
        page: params.page,
        page_size: params.page_size,
    };

    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Valuation Report Handler
// =============================================================================

/// Handler for GET /api/reports/inventory/valuation
///
/// Generates an inventory valuation report with stock values.
///
/// # Query Parameters
///
/// - `store_id` (optional): Filter by store
/// - `currency` (optional): Currency for the report (default: HNL)
///
/// # Response
///
/// - 200 OK: Valuation report with total values
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks reports:inventory permission
pub async fn get_valuation_report_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ValuationReportQueryParams>,
) -> Result<Json<ValuationReportResponse>, Response> {
    require_permission(&ctx, "reports:inventory")?;

    let use_case = GetValuationReportUseCase::new(
        state.stock_repo(),
        state.movement_repo(),
        state.product_repo(),
    );

    let query = ValuationReportQuery {
        store_id: params.store_id,
        currency: params.currency,
    };

    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Low Stock Report Handler
// =============================================================================

/// Handler for GET /api/reports/inventory/low-stock
///
/// Generates a low stock report with reorder suggestions.
///
/// # Query Parameters
///
/// - `store_id` (optional): Filter by store
/// - `include_zero_stock` (optional): Include items with zero stock (default: true)
///
/// # Response
///
/// - 200 OK: Low stock report with shortage and reorder suggestions
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks reports:inventory permission
pub async fn get_low_stock_report_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<LowStockReportQueryParams>,
) -> Result<Json<LowStockReportResponse>, Response> {
    require_permission(&ctx, "reports:inventory")?;

    let use_case = GetLowStockReportUseCase::new(state.stock_repo(), state.product_repo());

    let query = LowStockReportQuery {
        store_id: params.store_id,
        include_zero_stock: params.include_zero_stock,
    };

    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Movements Report Handler
// =============================================================================

/// Handler for GET /api/reports/inventory/movements
///
/// Generates a paginated movements report with filters.
///
/// # Query Parameters
///
/// - `store_id` (optional): Filter by store
/// - `stock_id` (optional): Filter by stock record
/// - `movement_type` (optional): Filter by movement type (in, out, adjustment, etc.)
/// - `from_date` (optional): Filter movements from this date
/// - `to_date` (optional): Filter movements to this date
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
///
/// # Response
///
/// - 200 OK: Paginated movements report
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks reports:inventory permission
pub async fn get_movements_report_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<MovementsReportQueryParams>,
) -> Result<Json<PaginatedResponse<MovementResponse>>, Response> {
    require_permission(&ctx, "reports:inventory")?;

    let use_case = GetMovementsReportUseCase::new(state.movement_repo());

    let query = MovementsReportQuery {
        store_id: params.store_id,
        stock_id: params.stock_id,
        movement_type: params.movement_type,
        from_date: params.from_date,
        to_date: params.to_date,
        page: params.page,
        page_size: params.page_size,
    };

    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
