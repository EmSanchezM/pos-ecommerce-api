// =============================================================================
// Goods Receipt Handlers
// =============================================================================
//
// These handlers implement the REST endpoints for goods receipt management:
// - POST /api/v1/goods-receipts - Create a goods receipt
// - GET /api/v1/goods-receipts - List goods receipts with pagination
// - GET /api/v1/goods-receipts/{id} - Get goods receipt details
// - PUT /api/v1/goods-receipts/{id}/confirm - Confirm goods receipt
// - PUT /api/v1/goods-receipts/{id}/cancel - Cancel goods receipt

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use purchasing::{
    CancelGoodsReceiptUseCase, ConfirmGoodsReceiptUseCase, CreateGoodsReceiptCommand,
    CreateGoodsReceiptUseCase, GetGoodsReceiptUseCase, GoodsReceiptDetailResponse,
    GoodsReceiptResponse, ListGoodsReceiptsQuery, ListGoodsReceiptsUseCase,
};
use inventory::PaginatedResponse;

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

// =============================================================================
// Query DTOs
// =============================================================================

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

/// Query parameters for listing goods receipts (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct ListGoodsReceiptsQueryParams {
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Filter by purchase order ID
    pub purchase_order_id: Option<Uuid>,
    /// Filter by status (draft, confirmed, cancelled)
    pub status: Option<String>,
}

impl From<ListGoodsReceiptsQueryParams> for ListGoodsReceiptsQuery {
    fn from(params: ListGoodsReceiptsQueryParams) -> Self {
        ListGoodsReceiptsQuery {
            store_id: params.store_id,
            purchase_order_id: params.purchase_order_id,
            status: params.status,
            page: params.page,
            page_size: params.page_size,
        }
    }
}

// =============================================================================
// Create Goods Receipt Handler
// =============================================================================

/// Handler for POST /api/v1/goods-receipts
///
/// Creates a new goods receipt for a purchase order.
///
/// # Request Body
///
/// ```json
/// {
///   "purchase_order_id": "uuid",
///   "store_id": "uuid",
///   "receipt_date": "2024-01-22",
///   "notes": "optional notes",
///   "items": [
///     {
///       "purchase_order_item_id": "uuid",
///       "product_id": "uuid",
///       "variant_id": "uuid (optional)",
///       "quantity_received": 10,
///       "unit_cost": 15.00,
///       "lot_number": "LOT001 (optional)",
///       "expiry_date": "2025-01-22 (optional)",
///       "notes": "optional item notes"
///     }
///   ]
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Goods receipt successfully created
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks goods_receipts:create permission
/// - 404 Not Found: Purchase order not found or not approved
pub async fn create_goods_receipt_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreateGoodsReceiptCommand>,
) -> Result<(StatusCode, Json<GoodsReceiptDetailResponse>), Response> {
    require_permission(&ctx, "goods_receipts:create")?;

    let use_case = CreateGoodsReceiptUseCase::new(
        state.goods_receipt_repo(),
        state.purchase_order_repo(),
    );

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

// =============================================================================
// List Goods Receipts Handler
// =============================================================================

/// Handler for GET /api/v1/goods-receipts
///
/// Lists goods receipts with pagination and optional filters.
///
/// # Query Parameters
///
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `store_id` (optional): Filter by store
/// - `purchase_order_id` (optional): Filter by purchase order
/// - `status` (optional): Filter by status (draft, confirmed, cancelled)
///
/// # Response
///
/// - 200 OK: Paginated list of goods receipts
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks goods_receipts:read permission
pub async fn list_goods_receipts_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListGoodsReceiptsQueryParams>,
) -> Result<Json<PaginatedResponse<GoodsReceiptResponse>>, Response> {
    require_permission(&ctx, "goods_receipts:read")?;

    let use_case = ListGoodsReceiptsUseCase::new(state.goods_receipt_repo());

    let query: ListGoodsReceiptsQuery = params.into();
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Goods Receipt Handler
// =============================================================================

/// Handler for GET /api/v1/goods-receipts/{id}
///
/// Gets detailed information about a specific goods receipt including all items.
///
/// # Path Parameters
///
/// - `id`: Goods Receipt UUID
///
/// # Response
///
/// - 200 OK: Goods receipt details with items
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks goods_receipts:read permission
/// - 404 Not Found: Goods receipt doesn't exist
pub async fn get_goods_receipt_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<GoodsReceiptDetailResponse>, Response> {
    require_permission(&ctx, "goods_receipts:read")?;

    let use_case = GetGoodsReceiptUseCase::new(state.goods_receipt_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Confirm Goods Receipt Handler
// =============================================================================

/// Handler for PUT /api/v1/goods-receipts/{id}/confirm
///
/// Confirms a draft goods receipt and updates inventory.
///
/// # Path Parameters
///
/// - `id`: Goods Receipt UUID
///
/// # Response
///
/// - 200 OK: Goods receipt successfully confirmed
/// - 400 Bad Request: Goods receipt is not in draft status
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks goods_receipts:confirm permission
/// - 404 Not Found: Goods receipt doesn't exist
pub async fn confirm_goods_receipt_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<GoodsReceiptDetailResponse>, Response> {
    require_permission(&ctx, "goods_receipts:confirm")?;

    let use_case = ConfirmGoodsReceiptUseCase::new(
        state.goods_receipt_repo(),
        state.purchase_order_repo(),
    );

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(id, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Cancel Goods Receipt Handler
// =============================================================================

/// Handler for PUT /api/v1/goods-receipts/{id}/cancel
///
/// Cancels a draft goods receipt.
///
/// # Path Parameters
///
/// - `id`: Goods Receipt UUID
///
/// # Response
///
/// - 200 OK: Goods receipt successfully cancelled
/// - 400 Bad Request: Goods receipt is not in draft status
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks goods_receipts:cancel permission
/// - 404 Not Found: Goods receipt doesn't exist
pub async fn cancel_goods_receipt_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<GoodsReceiptDetailResponse>, Response> {
    require_permission(&ctx, "goods_receipts:cancel")?;

    let use_case = CancelGoodsReceiptUseCase::new(state.goods_receipt_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
