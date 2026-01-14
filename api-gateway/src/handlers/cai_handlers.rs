// CAI HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for CAI (Código de Autorización de Impresión) management:
// - POST /terminals/:id/cai - Assign CAI to terminal (requires super_admin)
// - GET /terminals/:id/cai/status - Get CAI status
// - POST /terminals/:id/cai/next-number - Get next invoice number
// - GET /terminals/:id/cai/history - Get CAI history

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::NaiveDate;
use uuid::Uuid;

use pos_core::{
    AssignCaiCommand, AssignCaiUseCase, CaiHistoryItemResponse, CaiStatusResponse,
    GetCaiStatusUseCase, GetNextInvoiceNumberUseCase, GetTerminalDetailUseCase,
    NextInvoiceNumberResponse, TerminalId,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_super_admin;
use crate::state::AppState;

// =============================================================================
// Response DTOs
// =============================================================================

/// Response DTO for CAI assignment
#[derive(Debug, serde::Serialize)]
pub struct CaiAssignmentResponse {
    pub id: Uuid,
    pub terminal_id: Uuid,
    pub cai_number: String,
    pub range_start: i64,
    pub range_end: i64,
    pub current_number: i64,
    pub expiration_date: NaiveDate,
    pub is_exhausted: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<pos_core::CaiRange> for CaiAssignmentResponse {
    fn from(cai: pos_core::CaiRange) -> Self {
        Self {
            id: cai.id(),
            terminal_id: Uuid::nil(), // Will be set by handler
            cai_number: cai.cai_number().as_str().to_string(),
            range_start: cai.range_start(),
            range_end: cai.range_end(),
            current_number: cai.current_number(),
            expiration_date: cai.expiration_date(),
            is_exhausted: cai.is_exhausted_flag(),
            created_at: cai.created_at(),
        }
    }
}

// =============================================================================
// Assign CAI Handler
// =============================================================================

/// Handler for POST /terminals/:id/cai
///
/// Assigns a CAI range to a terminal. Requires super_admin role.
///
/// # Path Parameters
///
/// - `id`: Terminal UUID
///
/// # Request Body
///
/// ```json
/// {
///   "cai_number": "ABC123-DEF456-GHI789",
///   "range_start": 1,
///   "range_end": 1000,
///   "expiration_date": "2025-12-31"
/// }
/// ```
///
/// # Response
///
/// - 201 Created: CAI successfully assigned
/// - 400 Bad Request: Validation error (invalid CAI format, invalid range)
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User is not super_admin
/// - 404 Not Found: Terminal doesn't exist
/// - 409 Conflict: CAI range overlaps with existing active range
/// - 500 Internal Server Error: Unexpected error
///
/// - Assign CAI with number, expiration date, and range
/// - Validate no overlap with existing active ranges
pub async fn assign_cai_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(terminal_id): Path<Uuid>,
    Json(command): Json<AssignCaiRequest>,
) -> Result<(StatusCode, Json<CaiAssignmentResponse>), Response> {
    // Check super_admin permission
    require_super_admin(&ctx)?;

    // Build the full command with terminal_id from path
    let full_command = AssignCaiCommand {
        terminal_id,
        cai_number: command.cai_number,
        range_start: command.range_start,
        range_end: command.range_end,
        expiration_date: command.expiration_date,
    };

    let use_case = AssignCaiUseCase::new(state.terminal_repo(), state.audit_repo());

    let cai_range = use_case
        .execute(full_command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    let mut response = CaiAssignmentResponse::from(cai_range);
    response.terminal_id = terminal_id;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Request body for assigning a CAI (without terminal_id, which comes from path)
#[derive(Debug, serde::Deserialize)]
pub struct AssignCaiRequest {
    pub cai_number: String,
    pub range_start: i64,
    pub range_end: i64,
    pub expiration_date: NaiveDate,
}

// =============================================================================
// Get CAI Status Handler
// =============================================================================

/// Handler for GET /terminals/:id/cai/status
///
/// Gets the current CAI status for a terminal.
///
/// # Path Parameters
///
/// - `id`: Terminal UUID
///
/// # Response
///
/// - 200 OK: CAI status with optional expiration warning
/// - 400 Bad Request: No CAI assigned to terminal
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Terminal doesn't exist
/// - 500 Internal Server Error: Unexpected error
///
/// - Return current number, remaining, and expiration date
/// - Include warning if CAI expires within 30 days
pub async fn get_cai_status_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(terminal_id): Path<Uuid>,
) -> Result<Json<CaiStatusResponse>, Response> {
    let use_case = GetCaiStatusUseCase::new(state.terminal_repo());

    let terminal_id = TerminalId::from_uuid(terminal_id);
    let response = use_case
        .execute(terminal_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Next Invoice Number Handler
// =============================================================================

/// Handler for POST /terminals/:id/cai/next-number
///
/// Gets the next invoice number for a terminal. This operation is atomic
/// and will increment the counter.
///
/// # Path Parameters
///
/// - `id`: Terminal UUID
///
/// # Response
///
/// - 200 OK: Next invoice number with CAI info
/// - 400 Bad Request: Terminal inactive, no CAI assigned, CAI expired, or range exhausted
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Terminal doesn't exist
/// - 500 Internal Server Error: Unexpected error
///
/// - Return current number and increment counter atomically
/// - Reject if CAI is expired
/// - Reject if range is exhausted
pub async fn get_next_invoice_number_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(terminal_id): Path<Uuid>,
) -> Result<Json<NextInvoiceNumberResponse>, Response> {
    let use_case = GetNextInvoiceNumberUseCase::new(state.terminal_repo());

    let terminal_id = TerminalId::from_uuid(terminal_id);
    let response = use_case
        .execute(terminal_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get CAI History Handler
// =============================================================================

/// Handler for GET /terminals/:id/cai/history
///
/// Gets the complete CAI history for a terminal.
///
/// # Path Parameters
///
/// - `id`: Terminal UUID
///
/// # Response
///
/// - 200 OK: List of all CAI ranges (current and historical)
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Terminal doesn't exist
/// - 500 Internal Server Error: Unexpected error
///
/// - Return complete CAI history ordered by creation date
pub async fn get_cai_history_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(terminal_id): Path<Uuid>,
) -> Result<Json<Vec<CaiHistoryItemResponse>>, Response> {
    // We use GetTerminalDetailUseCase which already fetches CAI history
    let use_case = GetTerminalDetailUseCase::new(state.terminal_repo());

    let terminal_id = TerminalId::from_uuid(terminal_id);
    let terminal_detail = use_case
        .execute(terminal_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(terminal_detail.cai_history))
}
