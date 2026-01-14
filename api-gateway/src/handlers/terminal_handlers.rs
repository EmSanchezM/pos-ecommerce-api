// Terminal HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for terminal management:
// - POST /stores/:store_id/terminals - Create a new terminal (requires super_admin)
// - GET /stores/:store_id/terminals - List terminals for a store
// - GET /terminals/:id - Get terminal details
// - PUT /terminals/:id - Update terminal
// - POST /terminals/:id/activate - Activate terminal
// - POST /terminals/:id/deactivate - Deactivate terminal
//
// Requirements: 2.1, 2.6, 4.3, 4.4

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use identity::StoreId;
use pos_core::{
    CreateTerminalCommand, CreateTerminalUseCase, GetTerminalDetailUseCase, ListTerminalsQuery,
    ListTerminalsUseCase, PaginatedTerminalsResponse, SetTerminalActiveUseCase,
    TerminalDetailResponse, TerminalId, TerminalResponse, UpdateTerminalCommand,
    UpdateTerminalUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_super_admin;
use crate::state::AppState;

// =============================================================================
// Create Terminal Handler
// =============================================================================

/// Handler for POST /stores/:store_id/terminals
///
/// Creates a new terminal for a store. Requires super_admin role.
///
/// # Path Parameters
///
/// - `store_id`: Store UUID
///
/// # Request Body
///
/// ```json
/// {
///   "code": "TERM-001",
///   "name": "Main Terminal"
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Terminal successfully created
/// - 400 Bad Request: Validation error (invalid code format)
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User is not super_admin
/// - 404 Not Found: Store doesn't exist
/// - 409 Conflict: Terminal code already exists in store
/// - 500 Internal Server Error: Unexpected error
///
/// # Requirements
///
/// - Requirement 2.1: Create terminal for active store with unique code
/// - Requirement 2.4: Only super_admin can create terminals
pub async fn create_terminal_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(store_id): Path<Uuid>,
    Json(command): Json<CreateTerminalCommandRequest>,
) -> Result<(StatusCode, Json<TerminalResponse>), Response> {
    // Check super_admin permission (Requirement 2.4)
    require_super_admin(&ctx)?;

    // Build the full command with store_id from path
    let full_command = CreateTerminalCommand {
        store_id,
        code: command.code,
        name: command.name,
    };

    let use_case = CreateTerminalUseCase::new(
        state.terminal_repo(),
        state.store_repo(),
        state.audit_repo(),
    );

    let terminal = use_case
        .execute(full_command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(TerminalResponse::from(terminal))))
}

/// Request body for creating a terminal (without store_id, which comes from path)
#[derive(Debug, serde::Deserialize)]
pub struct CreateTerminalCommandRequest {
    pub code: String,
    pub name: String,
}

// =============================================================================
// List Terminals Handler
// =============================================================================

/// Handler for GET /stores/:store_id/terminals
///
/// Lists terminals for a store with pagination and optional filters.
///
/// # Path Parameters
///
/// - `store_id`: Store UUID
///
/// # Query Parameters
///
/// - `is_active` (optional): Filter by active status
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
///
/// # Response
///
/// - 200 OK: Paginated list of terminals with CAI status
/// - 401 Unauthorized: Missing or invalid token
/// - 500 Internal Server Error: Unexpected error
///
/// # Requirements
///
/// - Requirement 4.3: List terminals with CAI status
pub async fn list_terminals_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(store_id): Path<Uuid>,
    Query(query): Query<ListTerminalsQuery>,
) -> Result<Json<PaginatedTerminalsResponse>, Response> {
    let use_case = ListTerminalsUseCase::new(state.terminal_repo());

    let store_id = StoreId::from_uuid(store_id);
    let response = use_case
        .execute(store_id, query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Terminal Detail Handler
// =============================================================================

/// Handler for GET /terminals/:id
///
/// Gets detailed information about a specific terminal, including
/// complete CAI history.
///
/// # Path Parameters
///
/// - `id`: Terminal UUID
///
/// # Response
///
/// - 200 OK: Terminal details with CAI history
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Terminal doesn't exist
/// - 500 Internal Server Error: Unexpected error
///
/// # Requirements
///
/// - Requirement 4.4: Get terminal details with complete CAI history
pub async fn get_terminal_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TerminalDetailResponse>, Response> {
    let use_case = GetTerminalDetailUseCase::new(state.terminal_repo());

    let terminal_id = TerminalId::from_uuid(id);
    let response = use_case
        .execute(terminal_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Update Terminal Handler
// =============================================================================

/// Handler for PUT /terminals/:id
///
/// Updates an existing terminal's details.
///
/// # Path Parameters
///
/// - `id`: Terminal UUID
///
/// # Request Body
///
/// ```json
/// {
///   "name": "New Terminal Name"
/// }
/// ```
///
/// All fields are optional - only provided fields will be updated.
///
/// # Response
///
/// - 200 OK: Terminal successfully updated
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Terminal doesn't exist
/// - 500 Internal Server Error: Unexpected error
pub async fn update_terminal_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<UpdateTerminalCommand>,
) -> Result<Json<TerminalResponse>, Response> {
    let use_case = UpdateTerminalUseCase::new(state.terminal_repo(), state.audit_repo());

    let terminal_id = TerminalId::from_uuid(id);
    let terminal = use_case
        .execute(terminal_id, command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(TerminalResponse::from(terminal)))
}

// =============================================================================
// Activate Terminal Handler
// =============================================================================

/// Handler for POST /terminals/:id/activate
///
/// Activates a terminal.
///
/// # Path Parameters
///
/// - `id`: Terminal UUID
///
/// # Response
///
/// - 200 OK: Terminal successfully activated
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Terminal doesn't exist
/// - 500 Internal Server Error: Unexpected error
///
/// # Requirements
///
/// - Requirement 2.6: Activate terminal
pub async fn activate_terminal_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TerminalResponse>, Response> {
    let use_case = SetTerminalActiveUseCase::new(state.terminal_repo(), state.audit_repo());

    let terminal_id = TerminalId::from_uuid(id);
    let terminal = use_case
        .activate(terminal_id, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(TerminalResponse::from(terminal)))
}

// =============================================================================
// Deactivate Terminal Handler
// =============================================================================

/// Handler for POST /terminals/:id/deactivate
///
/// Deactivates a terminal. CAI history is preserved.
///
/// # Path Parameters
///
/// - `id`: Terminal UUID
///
/// # Response
///
/// - 200 OK: Terminal successfully deactivated
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Terminal doesn't exist
/// - 500 Internal Server Error: Unexpected error
///
/// # Requirements
///
/// - Requirement 2.6: Deactivate terminal preserving CAI history
pub async fn deactivate_terminal_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TerminalResponse>, Response> {
    let use_case = SetTerminalActiveUseCase::new(state.terminal_repo(), state.audit_repo());

    let terminal_id = TerminalId::from_uuid(id);
    let terminal = use_case
        .deactivate(terminal_id, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(TerminalResponse::from(terminal)))
}
