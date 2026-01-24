// =============================================================================
// Adjustment Handlers
// =============================================================================
//
// These handlers implement the REST endpoints for stock adjustments:
// - POST /api/inventory/adjustments - Create an adjustment
// - GET /api/inventory/adjustments - List adjustments with pagination
// - GET /api/inventory/adjustments/{id} - Get adjustment details
// - PUT /api/inventory/adjustments/{id}/submit - Submit for approval
// - PUT /api/inventory/adjustments/{id}/approve - Approve an adjustment
// - PUT /api/inventory/adjustments/{id}/reject - Reject an adjustment
// - POST /api/inventory/adjustments/{id}/apply - Apply an approved adjustment

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use inventory::{
    ApplyAdjustmentCommand, ApplyAdjustmentUseCase, ApproveAdjustmentCommand,
    ApproveAdjustmentUseCase, CreateAdjustmentCommand, CreateAdjustmentUseCase,
    GetAdjustmentUseCase, ListAdjustmentsQuery, ListAdjustmentsUseCase, PaginatedResponse,
    SubmitAdjustmentCommand, SubmitAdjustmentUseCase, AdjustmentDetailResponse, AdjustmentResponse,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

use super::products::{default_page, default_page_size};

// =============================================================================
// Query DTOs
// =============================================================================

/// Query parameters for listing adjustments (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct ListAdjustmentsQueryParams {
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Filter by status (draft, pending_approval, approved, rejected, applied)
    pub status: Option<String>,
}

impl From<ListAdjustmentsQueryParams> for ListAdjustmentsQuery {
    fn from(params: ListAdjustmentsQueryParams) -> Self {
        ListAdjustmentsQuery {
            store_id: params.store_id,
            status: params.status,
            page: params.page,
            page_size: params.page_size,
        }
    }
}

/// Request body for approving/rejecting an adjustment
#[derive(Debug, Deserialize)]
pub struct ApproveRejectRequest {
    /// Notes from the approver (optional)
    pub notes: Option<String>,
}

// =============================================================================
// Create Adjustment Handler
// =============================================================================

/// Handler for POST /api/inventory/adjustments
///
/// Creates a new stock adjustment in draft status.
///
/// # Request Body
///
/// ```json
/// {
///   "store_id": "uuid",
///   "adjustment_type": "increase|decrease",
///   "adjustment_reason": "damage|theft|loss|found|correction|expiration",
///   "notes": "optional notes",
///   "items": [
///     {
///       "stock_id": "uuid",
///       "quantity": 10,
///       "unit_cost": 5.00,
///       "notes": "optional item notes"
///     }
///   ]
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Adjustment successfully created
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks adjustments:create permission
pub async fn create_adjustment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreateAdjustmentCommand>,
) -> Result<(StatusCode, Json<AdjustmentDetailResponse>), Response> {
    require_permission(&ctx, "adjustments:create")?;

    let use_case = CreateAdjustmentUseCase::new(state.adjustment_repo());

    let actor_id = ctx.user_id().clone();
    let response = use_case
        .execute(command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

// =============================================================================
// List Adjustments Handler
// =============================================================================

/// Handler for GET /api/inventory/adjustments
///
/// Lists adjustments with pagination and optional filters.
///
/// # Query Parameters
///
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `store_id` (optional): Filter by store
/// - `status` (optional): Filter by status (draft, pending_approval, approved, rejected, applied)
///
/// # Response
///
/// - 200 OK: Paginated list of adjustments
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:adjustments:read permission
pub async fn list_adjustments_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListAdjustmentsQueryParams>,
) -> Result<Json<PaginatedResponse<AdjustmentResponse>>, Response> {
    require_permission(&ctx, "adjustments:read")?;

    let use_case = ListAdjustmentsUseCase::new(state.adjustment_repo());

    let query: ListAdjustmentsQuery = params.into();
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Adjustment Handler
// =============================================================================

/// Handler for GET /api/inventory/adjustments/{id}
///
/// Gets detailed information about a specific adjustment including all items.
///
/// # Path Parameters
///
/// - `id`: Adjustment UUID
///
/// # Response
///
/// - 200 OK: Adjustment details with items
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:adjustments:read permission
/// - 404 Not Found: Adjustment doesn't exist
pub async fn get_adjustment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AdjustmentDetailResponse>, Response> {
    require_permission(&ctx, "adjustments:read")?;

    let use_case = GetAdjustmentUseCase::new(state.adjustment_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Submit Adjustment Handler
// =============================================================================

/// Handler for PUT /api/inventory/adjustments/{id}/submit
///
/// Submits a draft adjustment for approval.
///
/// # Path Parameters
///
/// - `id`: Adjustment UUID
///
/// # Response
///
/// - 200 OK: Adjustment successfully submitted
/// - 400 Bad Request: Adjustment is not in draft status or has no items
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:adjustments:submit permission
/// - 404 Not Found: Adjustment doesn't exist
pub async fn submit_adjustment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AdjustmentDetailResponse>, Response> {
    require_permission(&ctx, "adjustments:submit")?;

    let use_case = SubmitAdjustmentUseCase::new(state.adjustment_repo());

    let command = SubmitAdjustmentCommand { adjustment_id: id };
    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Approve Adjustment Handler
// =============================================================================

/// Handler for PUT /api/inventory/adjustments/{id}/approve
///
/// Approves a pending adjustment.
///
/// # Path Parameters
///
/// - `id`: Adjustment UUID
///
/// # Request Body (optional)
///
/// ```json
/// {
///   "notes": "Approval notes"
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Adjustment successfully approved
/// - 400 Bad Request: Adjustment is not in pending_approval status
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks adjustments:approve permission
/// - 404 Not Found: Adjustment doesn't exist
pub async fn approve_adjustment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<Option<ApproveRejectRequest>>,
) -> Result<Json<AdjustmentDetailResponse>, Response> {
    require_permission(&ctx, "adjustments:approve")?;

    let use_case = ApproveAdjustmentUseCase::new(state.adjustment_repo());

    let command = ApproveAdjustmentCommand {
        adjustment_id: id,
        approve: true,
        notes: body.and_then(|b| b.notes),
    };
    let approver_id = ctx.user_id().clone();

    let response = use_case
        .execute(command, approver_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Reject Adjustment Handler
// =============================================================================

/// Handler for PUT /api/inventory/adjustments/{id}/reject
///
/// Rejects a pending adjustment.
///
/// # Path Parameters
///
/// - `id`: Adjustment UUID
///
/// # Request Body (optional)
///
/// ```json
/// {
///   "notes": "Rejection reason"
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Adjustment successfully rejected
/// - 400 Bad Request: Adjustment is not in pending_approval status
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks adjustments:reject permission
/// - 404 Not Found: Adjustment doesn't exist
pub async fn reject_adjustment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<Option<ApproveRejectRequest>>,
) -> Result<Json<AdjustmentDetailResponse>, Response> {
    require_permission(&ctx, "adjustments:reject")?;

    let use_case = ApproveAdjustmentUseCase::new(state.adjustment_repo());

    let command = ApproveAdjustmentCommand {
        adjustment_id: id,
        approve: false,
        notes: body.and_then(|b| b.notes),
    };
    let approver_id = ctx.user_id().clone();

    let response = use_case
        .execute(command, approver_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Apply Adjustment Handler
// =============================================================================

/// Handler for POST /api/inventory/adjustments/{id}/apply
///
/// Applies an approved adjustment to inventory stock.
///
/// # Path Parameters
///
/// - `id`: Adjustment UUID
///
/// # Response
///
/// - 200 OK: Adjustment successfully applied to stock
/// - 400 Bad Request: Adjustment is not in approved status
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks adjustments:apply permission
/// - 404 Not Found: Adjustment doesn't exist
pub async fn apply_adjustment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AdjustmentDetailResponse>, Response> {
    require_permission(&ctx, "adjustments:apply")?;

    let use_case = ApplyAdjustmentUseCase::new(
        state.adjustment_repo(),
        state.stock_repo(),
        state.movement_repo(),
    );

    let command = ApplyAdjustmentCommand { adjustment_id: id };
    let actor_id = ctx.user_id().clone();

    let response = use_case
        .execute(command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
