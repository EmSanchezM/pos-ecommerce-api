// Store HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for store management:
// - POST /stores - Create a new store (requires super_admin)
// - GET /stores - List stores with pagination and filters
// - GET /stores/:id - Get store details
// - PUT /stores/:id - Update store
// - POST /stores/:id/activate - Activate store (requires super_admin)
// - POST /stores/:id/deactivate - Deactivate store (requires super_admin)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use identity::{
    CreateStoreCommand, CreateStoreUseCase, StoreId, UpdateStoreCommand, UpdateStoreUseCase,
};
use pos_core::{
    GetStoreDetailUseCase, ListStoresQuery, ListStoresUseCase, PaginatedStoresResponse,
    SetStoreActiveUseCaseExtended, StoreDetailResponse,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_super_admin;
use crate::state::AppState;

// =============================================================================
// Response DTOs
// =============================================================================

/// Response DTO for store creation/update operations
#[derive(Debug, serde::Serialize)]
pub struct StoreResponse {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub is_ecommerce: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<identity::Store> for StoreResponse {
    fn from(store: identity::Store) -> Self {
        Self {
            id: store.id().into_uuid(),
            name: store.name().to_string(),
            address: store.address().to_string(),
            is_ecommerce: store.is_ecommerce(),
            is_active: store.is_active(),
            created_at: store.created_at(),
            updated_at: store.updated_at(),
        }
    }
}

// =============================================================================
// Create Store Handler
// =============================================================================

/// Handler for POST /stores
///
/// Creates a new store. Requires super_admin role.
///
/// # Request Body
///
/// ```json
/// {
///   "name": "Store Name",
///   "address": "123 Main St",
///   "is_ecommerce": false
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Store successfully created
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User is not super_admin
/// - 500 Internal Server Error: Unexpected error
///
/// - Create store with name, address, and type
/// - Only super_admin can create stores
pub async fn create_store_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreateStoreCommand>,
) -> Result<(StatusCode, Json<StoreResponse>), Response> {
    // Check super_admin permission (Requirement 1.5)
    require_super_admin(&ctx)?;

    let use_case = CreateStoreUseCase::new(state.store_repo(), state.audit_repo());

    let store = use_case
        .execute(command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(StoreResponse::from(store))))
}

// =============================================================================
// List Stores Handler
// =============================================================================

/// Handler for GET /stores
///
/// Lists stores with pagination and optional filters.
/// Returns only stores the user has access to based on their roles.
///
/// # Query Parameters
///
/// - `is_active` (optional): Filter by active status
/// - `is_ecommerce` (optional): Filter by e-commerce type
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
///
/// # Response
///
/// - 200 OK: Paginated list of stores
/// - 401 Unauthorized: Missing or invalid token
/// - 500 Internal Server Error: Unexpected error
///
/// - List stores with pagination and filters
/// - Filter by user access (TODO: implement access filtering)
pub async fn list_stores_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Query(query): Query<ListStoresQuery>,
) -> Result<Json<PaginatedStoresResponse>, Response> {
    let use_case = ListStoresUseCase::new(state.store_repo());

    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Store Detail Handler
// =============================================================================

/// Handler for GET /stores/:id
///
/// Gets detailed information about a specific store, including
/// the count of active terminals.
///
/// # Path Parameters
///
/// - `id`: Store UUID
///
/// # Response
///
/// - 200 OK: Store details with terminal count
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Store doesn't exist
/// - 500 Internal Server Error: Unexpected error
///
/// - Get store details with active terminal count
pub async fn get_store_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<StoreDetailResponse>, Response> {
    let use_case = GetStoreDetailUseCase::new(state.store_repo(), state.terminal_repo());

    let store_id = StoreId::from_uuid(id);
    let response = use_case
        .execute(store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Update Store Handler
// =============================================================================

/// Handler for PUT /stores/:id
///
/// Updates an existing store's details.
///
/// # Path Parameters
///
/// - `id`: Store UUID
///
/// # Request Body
///
/// ```json
/// {
///   "name": "New Store Name",
///   "address": "456 New St",
///   "is_ecommerce": true
/// }
/// ```
///
/// All fields are optional - only provided fields will be updated.
///
/// # Response
///
/// - 200 OK: Store successfully updated
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Store doesn't exist
/// - 500 Internal Server Error: Unexpected error
/// 
/// - Update store details
pub async fn update_store_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<UpdateStoreCommand>,
) -> Result<Json<StoreResponse>, Response> {
    let use_case = UpdateStoreUseCase::new(state.store_repo(), state.audit_repo());

    let store_id = StoreId::from_uuid(id);
    let store = use_case
        .execute(store_id, command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(StoreResponse::from(store)))
}

// =============================================================================
// Activate Store Handler
// =============================================================================

/// Handler for POST /stores/:id/activate
///
/// Activates a store. Requires super_admin role.
/// Note: Reactivating a store does NOT automatically reactivate its terminals.
///
/// # Path Parameters
///
/// - `id`: Store UUID
///
/// # Response
///
/// - 200 OK: Store successfully activated
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User is not super_admin
/// - 404 Not Found: Store doesn't exist
/// - 500 Internal Server Error: Unexpected error
///
/// - Reactivate store without cascading to terminals
/// - Only super_admin can activate stores
pub async fn activate_store_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<StoreResponse>, Response> {
    // Check super_admin permission (Requirement 1.5)
    require_super_admin(&ctx)?;

    let use_case = SetStoreActiveUseCaseExtended::new(
        state.store_repo(),
        state.terminal_repo(),
        state.audit_repo(),
    );

    let store_id = StoreId::from_uuid(id);
    let store = use_case
        .activate(store_id, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(StoreResponse::from(store)))
}

// =============================================================================
// Deactivate Store Handler
// =============================================================================

/// Handler for POST /stores/:id/deactivate
///
/// Deactivates a store. Requires super_admin role.
/// When a store is deactivated, all its terminals are also deactivated.
///
/// # Path Parameters
///
/// - `id`: Store UUID
///
/// # Response
///
/// - 200 OK: Store successfully deactivated
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User is not super_admin
/// - 404 Not Found: Store doesn't exist
/// - 500 Internal Server Error: Unexpected error
///
/// - Deactivate store and cascade to terminals
/// - Only super_admin can deactivate stores
pub async fn deactivate_store_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<StoreResponse>, Response> {
    // Check super_admin permission (Requirement 1.5)
    require_super_admin(&ctx)?;

    let use_case = SetStoreActiveUseCaseExtended::new(
        state.store_repo(),
        state.terminal_repo(),
        state.audit_repo(),
    );

    let store_id = StoreId::from_uuid(id);
    let store = use_case
        .deactivate(store_id, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(StoreResponse::from(store)))
}
