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
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use inventory::{
    Currency, InventoryMovement, InventoryMovementRepository, InventoryStock,
    InventoryStockRepository, MovementType, PaginatedResponse,
};
use purchasing::{
    CancelGoodsReceiptUseCase, CreateGoodsReceiptCommand, CreateGoodsReceiptUseCase,
    GetGoodsReceiptUseCase, GoodsReceiptDetailResponse, GoodsReceiptItemResponse,
    GoodsReceiptRepository, GoodsReceiptResponse, ListGoodsReceiptsQuery, ListGoodsReceiptsUseCase,
    PgGoodsReceiptRepository, PgPurchaseOrderRepository, PurchaseOrderRepository, PurchasingError,
};

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::org_scope::verify_store_in_org;
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
    JsonBody(command): JsonBody<CreateGoodsReceiptCommand>,
) -> Result<(StatusCode, Json<GoodsReceiptDetailResponse>), Response> {
    require_permission(&ctx, "goods_receipts:create")?;
    verify_store_in_org(state.pool(), &ctx, command.store_id).await?;

    let use_case =
        CreateGoodsReceiptUseCase::new(state.goods_receipt_repo(), state.purchase_order_repo());

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
    if let Some(sid) = params.store_id {
        verify_store_in_org(state.pool(), &ctx, sid).await?;
    }

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

    let actor_id = *ctx.user_id();
    let receipt_id = purchasing::GoodsReceiptId::from_uuid(id);

    // Read receipt
    let mut receipt = state
        .goods_receipt_repo()
        .find_by_id_with_items(receipt_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?
        .ok_or_else(|| AppError::from(PurchasingError::GoodsReceiptNotFound(id)).into_response())?;

    // Domain logic: confirm receipt
    receipt
        .confirm(actor_id)
        .map_err(|e| AppError::from(e).into_response())?;

    // Read purchase order
    let mut order = state
        .purchase_order_repo()
        .find_by_id_with_items(receipt.purchase_order_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?
        .ok_or_else(|| {
            AppError::from(PurchasingError::PurchaseOrderNotFound(
                receipt.purchase_order_id().into_uuid(),
            ))
            .into_response()
        })?;

    // Domain logic: update received quantities
    for receipt_item in receipt.items() {
        for order_item in order.items_mut() {
            if order_item.id() == receipt_item.purchase_order_item_id() {
                order_item.add_received_quantity(receipt_item.quantity_received());
                break;
            }
        }
    }

    if order.all_items_received() {
        order
            .receive_complete(actor_id, receipt.receipt_date())
            .map_err(|e| AppError::from(e).into_response())?;
    } else if order.has_received_items() {
        order
            .receive_partial(actor_id)
            .map_err(|e| AppError::from(e).into_response())?;
    }

    // All writes in a single transaction
    let mut tx = state
        .pool()
        .begin()
        .await
        .map_err(|e| AppError::from(PurchasingError::from(e)).into_response())?;

    PgGoodsReceiptRepository::update_in_tx(&mut tx, &receipt)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    PgPurchaseOrderRepository::update_in_tx(&mut tx, &order)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    tx.commit()
        .await
        .map_err(|e| AppError::from(PurchasingError::from(e)).into_response())?;

    // Update inventory stock for each receipt item
    let receipt_store_id = receipt.store_id();
    let receipt_uuid = receipt.id().into_uuid();
    let stock_repo = state.stock_repo();
    let movement_repo = state.movement_repo();

    for receipt_item in receipt.items() {
        let product_id = receipt_item.product_id();
        let variant_id = receipt_item.variant_id();
        let quantity = receipt_item.quantity_received();
        let unit_cost = Some(receipt_item.unit_cost());

        // Find or create stock record at the store
        let existing = if let Some(vid) = variant_id {
            stock_repo
                .find_by_store_and_variant(receipt_store_id, vid)
                .await
                .map_err(|e| AppError::from(e).into_response())?
        } else {
            stock_repo
                .find_by_store_and_product(receipt_store_id, product_id)
                .await
                .map_err(|e| AppError::from(e).into_response())?
        };

        let mut stock = match existing {
            Some(s) => s,
            None => {
                let new_stock = if let Some(vid) = variant_id {
                    InventoryStock::create_for_variant(receipt_store_id, vid)
                        .map_err(|e| AppError::from(e).into_response())?
                } else {
                    InventoryStock::create_for_product(receipt_store_id, product_id)
                        .map_err(|e| AppError::from(e).into_response())?
                };
                stock_repo
                    .save(&new_stock)
                    .await
                    .map_err(|e| AppError::from(e).into_response())?;
                new_stock
            }
        };

        let expected_version = stock.version();
        stock
            .adjust_quantity(quantity)
            .map_err(|e| AppError::from(e).into_response())?;
        stock.increment_version();

        stock_repo
            .update_with_version(&stock, expected_version)
            .await
            .map_err(|e| AppError::from(e).into_response())?;

        // Create inventory movement for goods receipt
        let movement = InventoryMovement::create(
            stock.id(),
            MovementType::In,
            Some("Goods receipt confirmed".to_string()),
            quantity,
            unit_cost,
            Currency::hnl(),
            stock.quantity(),
            Some("goods_receipt".to_string()),
            Some(receipt_uuid),
            actor_id,
            None,
        );
        movement_repo
            .save(&movement)
            .await
            .map_err(|e| AppError::from(e).into_response())?;
    }

    // Build response
    let items: Vec<GoodsReceiptItemResponse> = receipt
        .items()
        .iter()
        .map(|item| GoodsReceiptItemResponse {
            id: item.id().into_uuid(),
            goods_receipt_id: item.goods_receipt_id().into_uuid(),
            purchase_order_item_id: item.purchase_order_item_id().into_uuid(),
            product_id: item.product_id().into_uuid(),
            variant_id: item.variant_id().map(|v| v.into_uuid()),
            quantity_received: item.quantity_received(),
            unit_cost: item.unit_cost(),
            lot_number: item.lot_number().map(|s| s.to_string()),
            expiry_date: item.expiry_date(),
            notes: item.notes().map(|s| s.to_string()),
        })
        .collect();

    Ok(Json(GoodsReceiptDetailResponse {
        id: receipt.id().into_uuid(),
        receipt_number: receipt.receipt_number().to_string(),
        purchase_order_id: receipt.purchase_order_id().into_uuid(),
        store_id: receipt.store_id().into_uuid(),
        receipt_date: receipt.receipt_date(),
        status: receipt.status().to_string(),
        notes: receipt.notes().map(|s| s.to_string()),
        received_by_id: receipt.received_by_id().into_uuid(),
        confirmed_by_id: receipt.confirmed_by_id().map(|id| id.into_uuid()),
        confirmed_at: receipt.confirmed_at(),
        items,
        created_at: receipt.created_at(),
        updated_at: receipt.updated_at(),
    }))
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
