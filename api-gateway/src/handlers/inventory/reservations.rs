// =============================================================================
// Reservation Handlers
// =============================================================================
//
// These handlers implement the REST endpoints for inventory reservations:
// - POST /api/inventory/reservations - Create a reservation
// - GET /api/inventory/reservations - List reservations with pagination
// - PUT /api/inventory/reservations/{id}/confirm - Confirm a reservation
// - PUT /api/inventory/reservations/{id}/cancel - Cancel a reservation
// - POST /api/inventory/reservations/expire - Expire all expired reservations

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use identity::ErrorResponse;
use inventory::{
    CancelReservationCommand, CancelReservationUseCase, ConfirmReservationCommand,
    ConfirmReservationUseCase, CreateReservationCommand, CreateReservationUseCase,
    ExpireReservationsResult, ExpireReservationsUseCase, ListReservationsQuery,
    ListReservationsUseCase, PaginatedResponse, ReservationResponse,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

use super::products::{default_page, default_page_size};

// =============================================================================
// Query DTOs
// =============================================================================

/// Query parameters for listing reservations (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct ListReservationsQueryParams {
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filter by stock ID
    pub stock_id: Option<Uuid>,
    /// Filter by status (pending, confirmed, cancelled, expired)
    pub status: Option<String>,
    /// Filter by reference type (cart, order, quote)
    pub reference_type: Option<String>,
}

impl From<ListReservationsQueryParams> for ListReservationsQuery {
    fn from(params: ListReservationsQueryParams) -> Self {
        ListReservationsQuery {
            stock_id: params.stock_id,
            status: params.status,
            reference_type: params.reference_type,
            page: params.page,
            page_size: params.page_size,
        }
    }
}

/// Response for the expire reservations batch operation
#[derive(Debug, Clone, Serialize)]
pub struct ExpireReservationsResponse {
    /// Number of reservations successfully expired
    pub expired_count: usize,
    /// Number of reservations that failed to expire
    pub failed_count: usize,
    /// Details of expired reservations
    pub expired_reservations: Vec<ReservationResponse>,
    /// Errors encountered during processing
    pub errors: Vec<String>,
}

impl From<ExpireReservationsResult> for ExpireReservationsResponse {
    fn from(result: ExpireReservationsResult) -> Self {
        Self {
            expired_count: result.expired_count,
            failed_count: result.failed_count,
            expired_reservations: result.expired_reservations,
            errors: result.errors,
        }
    }
}

/// Handler for POST /api/inventory/reservations
///
/// Creates a new inventory reservation for stock.
///
/// # Request Body
///
/// ```json
/// {
///   "stock_id": "uuid",
///   "reference_type": "cart",
///   "reference_id": "uuid",
///   "quantity": 10,
///   "expires_at": "2024-12-31T23:59:59Z"
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Reservation successfully created
/// - 400 Bad Request: Validation error or insufficient stock
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks cart:add or sales:create permission
/// - 404 Not Found: Stock record doesn't exist
pub async fn create_reservation_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreateReservationCommand>,
) -> Result<(StatusCode, Json<ReservationResponse>), Response> {
    // Check for cart:add OR sales:create permission
    let has_cart_add = ctx.has_permission("cart:add");
    let has_sales_create = ctx.has_permission("sales:create");

    if !has_cart_add && !has_sales_create {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            ErrorResponse::new("FORBIDDEN", "Requires cart:add or sales:create permission"),
        )
        .into_response());
    }

    let use_case = CreateReservationUseCase::new(state.stock_repo(), state.reservation_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Handler for GET /api/inventory/reservations
///
/// Lists reservations with pagination and optional filters.
///
/// # Query Parameters
///
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `stock_id` (optional): Filter by stock record
/// - `status` (optional): Filter by status (pending, confirmed, cancelled, expired)
/// - `reference_type` (optional): Filter by reference type (cart, order, quote)
///
/// # Response
///
/// - 200 OK: Paginated list of reservations
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
pub async fn list_reservations_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListReservationsQueryParams>,
) -> Result<Json<PaginatedResponse<ReservationResponse>>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = ListReservationsUseCase::new(state.reservation_repo());

    let query: ListReservationsQuery = params.into();
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/inventory/reservations/{id}/confirm
///
/// Confirms a pending reservation and decrements stock.
///
/// # Path Parameters
///
/// - `id`: Reservation UUID
///
/// # Response
///
/// - 200 OK: Reservation successfully confirmed
/// - 400 Bad Request: Reservation is not in pending status
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks sales:create permission
/// - 404 Not Found: Reservation doesn't exist
pub async fn confirm_reservation_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ReservationResponse>, Response> {
    require_permission(&ctx, "sales:create")?;

    let use_case = ConfirmReservationUseCase::new(
        state.reservation_repo(),
        state.stock_repo(),
        state.movement_repo(),
    );

    let command = ConfirmReservationCommand { reservation_id: id };
    let actor_id = *ctx.user_id();

    let response = use_case
        .execute(command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/inventory/reservations/{id}/cancel
///
/// Cancels a pending reservation and releases reserved stock.
///
/// # Path Parameters
///
/// - `id`: Reservation UUID
///
/// # Response
///
/// - 200 OK: Reservation successfully cancelled
/// - 400 Bad Request: Reservation is not in pending status
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks cart:remove or sales:void permission
/// - 404 Not Found: Reservation doesn't exist
pub async fn cancel_reservation_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ReservationResponse>, Response> {
    // Check for cart:remove OR sales:void permission
    let has_cart_remove = ctx.has_permission("cart:remove");
    let has_sales_void = ctx.has_permission("sales:void");

    if !has_cart_remove && !has_sales_void {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            ErrorResponse::new("FORBIDDEN", "Requires cart:remove or sales:void permission"),
        )
        .into_response());
    }

    let use_case = CancelReservationUseCase::new(state.reservation_repo(), state.stock_repo());

    let command = CancelReservationCommand { reservation_id: id };

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for POST /api/inventory/reservations/expire
///
/// Batch process to expire all pending reservations past their expiration time.
/// This is typically called by a scheduled job.
///
/// # Response
///
/// - 200 OK: Batch expiration completed with result summary
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks system:admin permission
pub async fn expire_reservations_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> Result<Json<ExpireReservationsResponse>, Response> {
    require_permission(&ctx, "system:admin")?;

    let use_case = ExpireReservationsUseCase::new(state.reservation_repo(), state.stock_repo());

    let result = use_case
        .execute()
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(ExpireReservationsResponse::from(result)))
}