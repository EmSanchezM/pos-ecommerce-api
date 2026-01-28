// =============================================================================
// Purchase Order Handlers
// =============================================================================
//
// These handlers implement the REST endpoints for purchase order management:
// - POST /api/v1/purchase-orders - Create a purchase order
// - GET /api/v1/purchase-orders - List purchase orders with pagination
// - GET /api/v1/purchase-orders/{id} - Get purchase order details
// - PUT /api/v1/purchase-orders/{id}/submit - Submit for approval
// - PUT /api/v1/purchase-orders/{id}/approve - Approve purchase order
// - PUT /api/v1/purchase-orders/{id}/reject - Reject purchase order
// - PUT /api/v1/purchase-orders/{id}/cancel - Cancel purchase order
// - PUT /api/v1/purchase-orders/{id}/close - Close purchase order

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use purchasing::{
    ApprovePurchaseOrderUseCase, CancelOrderCommand, CancelPurchaseOrderUseCase,
    ClosePurchaseOrderUseCase, CreatePurchaseOrderCommand, CreatePurchaseOrderUseCase,
    GetPurchaseOrderUseCase, ListPurchaseOrdersQuery, ListPurchaseOrdersUseCase,
    PurchaseOrderDetailResponse, PurchaseOrderResponse, RejectOrderCommand,
    RejectPurchaseOrderUseCase, SubmitPurchaseOrderUseCase,
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

/// Query parameters for listing purchase orders (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct ListPurchaseOrdersQueryParams {
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Filter by vendor ID
    pub vendor_id: Option<Uuid>,
    /// Filter by status (draft, submitted, approved, partially_received, received, closed, cancelled)
    pub status: Option<String>,
    /// Search by order number
    pub search: Option<String>,
}

impl From<ListPurchaseOrdersQueryParams> for ListPurchaseOrdersQuery {
    fn from(params: ListPurchaseOrdersQueryParams) -> Self {
        ListPurchaseOrdersQuery {
            store_id: params.store_id,
            vendor_id: params.vendor_id,
            status: params.status,
            search: params.search,
            page: params.page,
            page_size: params.page_size,
        }
    }
}

// =============================================================================
// Create Purchase Order Handler
// =============================================================================

/// Handler for POST /api/v1/purchase-orders
///
/// Creates a new purchase order in draft status.
///
/// # Request Body
///
/// ```json
/// {
///   "store_id": "uuid",
///   "vendor_id": "uuid",
///   "order_date": "2024-01-15",
///   "expected_delivery_date": "2024-01-22",
///   "currency": "HNL",
///   "payment_terms_days": 30,
///   "notes": "optional notes",
///   "items": [
///     {
///       "product_id": "uuid",
///       "variant_id": "uuid (optional)",
///       "description": "Product description",
///       "quantity_ordered": 10,
///       "unit_of_measure": "unit",
///       "unit_cost": 15.00,
///       "discount_percent": 0,
///       "tax_percent": 15
///     }
///   ]
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Purchase order successfully created
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks purchase_orders:create permission
/// - 404 Not Found: Vendor not found
pub async fn create_purchase_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreatePurchaseOrderCommand>,
) -> Result<(StatusCode, Json<PurchaseOrderDetailResponse>), Response> {
    require_permission(&ctx, "purchase_orders:create")?;

    let use_case = CreatePurchaseOrderUseCase::new(
        state.purchase_order_repo(),
        state.vendor_repo(),
    );

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

// =============================================================================
// List Purchase Orders Handler
// =============================================================================

/// Handler for GET /api/v1/purchase-orders
///
/// Lists purchase orders with pagination and optional filters.
///
/// # Query Parameters
///
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `store_id` (optional): Filter by store
/// - `vendor_id` (optional): Filter by vendor
/// - `status` (optional): Filter by status
/// - `search` (optional): Search by order number
///
/// # Response
///
/// - 200 OK: Paginated list of purchase orders
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks purchase_orders:read permission
pub async fn list_purchase_orders_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListPurchaseOrdersQueryParams>,
) -> Result<Json<PaginatedResponse<PurchaseOrderResponse>>, Response> {
    require_permission(&ctx, "purchase_orders:read")?;

    let use_case = ListPurchaseOrdersUseCase::new(state.purchase_order_repo());

    let query: ListPurchaseOrdersQuery = params.into();
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Purchase Order Handler
// =============================================================================

/// Handler for GET /api/v1/purchase-orders/{id}
///
/// Gets detailed information about a specific purchase order including all items.
///
/// # Path Parameters
///
/// - `id`: Purchase Order UUID
///
/// # Response
///
/// - 200 OK: Purchase order details with items
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks purchase_orders:read permission
/// - 404 Not Found: Purchase order doesn't exist
pub async fn get_purchase_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PurchaseOrderDetailResponse>, Response> {
    require_permission(&ctx, "purchase_orders:read")?;

    let use_case = GetPurchaseOrderUseCase::new(state.purchase_order_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Submit Purchase Order Handler
// =============================================================================

/// Handler for PUT /api/v1/purchase-orders/{id}/submit
///
/// Submits a draft purchase order for approval.
///
/// # Path Parameters
///
/// - `id`: Purchase Order UUID
///
/// # Response
///
/// - 200 OK: Purchase order successfully submitted
/// - 400 Bad Request: Purchase order is not in draft status or has no items
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks purchase_orders:submit permission
/// - 404 Not Found: Purchase order doesn't exist
pub async fn submit_purchase_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PurchaseOrderDetailResponse>, Response> {
    require_permission(&ctx, "purchase_orders:submit")?;

    let use_case = SubmitPurchaseOrderUseCase::new(state.purchase_order_repo());

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(id, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Approve Purchase Order Handler
// =============================================================================

/// Handler for PUT /api/v1/purchase-orders/{id}/approve
///
/// Approves a submitted purchase order.
///
/// # Path Parameters
///
/// - `id`: Purchase Order UUID
///
/// # Response
///
/// - 200 OK: Purchase order successfully approved
/// - 400 Bad Request: Purchase order is not in submitted status or self-approval attempted
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks purchase_orders:approve permission
/// - 404 Not Found: Purchase order doesn't exist
pub async fn approve_purchase_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PurchaseOrderDetailResponse>, Response> {
    require_permission(&ctx, "purchase_orders:approve")?;

    let use_case = ApprovePurchaseOrderUseCase::new(state.purchase_order_repo());

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(id, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Reject Purchase Order Handler
// =============================================================================

/// Handler for PUT /api/v1/purchase-orders/{id}/reject
///
/// Rejects a submitted purchase order (returns to draft status).
///
/// # Path Parameters
///
/// - `id`: Purchase Order UUID
///
/// # Request Body (optional)
///
/// ```json
/// {
///   "reason": "Rejection reason"
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Purchase order successfully rejected
/// - 400 Bad Request: Purchase order is not in submitted status
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks purchase_orders:approve permission
/// - 404 Not Found: Purchase order doesn't exist
pub async fn reject_purchase_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<Option<RejectOrderCommand>>,
) -> Result<Json<PurchaseOrderDetailResponse>, Response> {
    require_permission(&ctx, "purchase_orders:approve")?;

    let use_case = RejectPurchaseOrderUseCase::new(state.purchase_order_repo());

    let command = body.unwrap_or(RejectOrderCommand { reason: None });
    let response = use_case
        .execute(id, command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Cancel Purchase Order Handler
// =============================================================================

/// Handler for PUT /api/v1/purchase-orders/{id}/cancel
///
/// Cancels a purchase order (only draft or submitted orders can be cancelled).
///
/// # Path Parameters
///
/// - `id`: Purchase Order UUID
///
/// # Request Body
///
/// ```json
/// {
///   "reason": "Cancellation reason"
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Purchase order successfully cancelled
/// - 400 Bad Request: Purchase order cannot be cancelled in current status
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks purchase_orders:cancel permission
/// - 404 Not Found: Purchase order doesn't exist
pub async fn cancel_purchase_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<CancelOrderCommand>,
) -> Result<Json<PurchaseOrderDetailResponse>, Response> {
    require_permission(&ctx, "purchase_orders:cancel")?;

    let use_case = CancelPurchaseOrderUseCase::new(state.purchase_order_repo());

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(id, command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Close Purchase Order Handler
// =============================================================================

/// Handler for PUT /api/v1/purchase-orders/{id}/close
///
/// Closes a fully received purchase order.
///
/// # Path Parameters
///
/// - `id`: Purchase Order UUID
///
/// # Response
///
/// - 200 OK: Purchase order successfully closed
/// - 400 Bad Request: Purchase order is not fully received
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks purchase_orders:close permission
/// - 404 Not Found: Purchase order doesn't exist
pub async fn close_purchase_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PurchaseOrderDetailResponse>, Response> {
    require_permission(&ctx, "purchase_orders:close")?;

    let use_case = ClosePurchaseOrderUseCase::new(state.purchase_order_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
