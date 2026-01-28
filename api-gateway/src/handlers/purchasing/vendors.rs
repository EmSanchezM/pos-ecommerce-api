// =============================================================================
// Vendor Handlers
// =============================================================================
//
// These handlers implement the REST endpoints for vendor management:
// - POST /api/v1/vendors - Create a vendor
// - GET /api/v1/vendors - List vendors with pagination
// - GET /api/v1/vendors/{id} - Get vendor details
// - PUT /api/v1/vendors/{id} - Update vendor
// - PUT /api/v1/vendors/{id}/activate - Activate a vendor
// - PUT /api/v1/vendors/{id}/deactivate - Deactivate a vendor

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use purchasing::{
    CreateVendorCommand, CreateVendorUseCase, GetVendorUseCase,
    ListVendorsQuery, ListVendorsUseCase, ToggleVendorStatusUseCase,
    UpdateVendorCommand, UpdateVendorUseCase, VendorResponse,
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

/// Query parameters for listing vendors (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct ListVendorsQueryParams {
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Search by name or code
    pub search: Option<String>,
}

impl From<ListVendorsQueryParams> for ListVendorsQuery {
    fn from(params: ListVendorsQueryParams) -> Self {
        ListVendorsQuery {
            is_active: params.is_active,
            search: params.search,
            page: params.page,
            page_size: params.page_size,
        }
    }
}

// =============================================================================
// Create Vendor Handler
// =============================================================================

/// Handler for POST /api/v1/vendors
///
/// Creates a new vendor.
///
/// # Request Body
///
/// ```json
/// {
///   "code": "VENDOR001",
///   "name": "Vendor Name",
///   "legal_name": "Vendor Legal Name Inc.",
///   "tax_id": "RTN12345678",
///   "email": "vendor@example.com",
///   "phone": "+504 1234-5678",
///   "address": "123 Main St",
///   "payment_terms_days": 30,
///   "currency": "HNL",
///   "notes": "Optional notes"
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Vendor successfully created
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks vendors:create permission
/// - 409 Conflict: Duplicate vendor code or tax ID
pub async fn create_vendor_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreateVendorCommand>,
) -> Result<(StatusCode, Json<VendorResponse>), Response> {
    require_permission(&ctx, "vendors:create")?;

    let use_case = CreateVendorUseCase::new(state.vendor_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

// =============================================================================
// List Vendors Handler
// =============================================================================

/// Handler for GET /api/v1/vendors
///
/// Lists vendors with pagination and optional filters.
///
/// # Query Parameters
///
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `is_active` (optional): Filter by active status
/// - `search` (optional): Search in name or code
///
/// # Response
///
/// - 200 OK: Paginated list of vendors
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks vendors:read permission
pub async fn list_vendors_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListVendorsQueryParams>,
) -> Result<Json<PaginatedResponse<VendorResponse>>, Response> {
    require_permission(&ctx, "vendors:read")?;

    let use_case = ListVendorsUseCase::new(state.vendor_repo());

    let query: ListVendorsQuery = params.into();
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Vendor Handler
// =============================================================================

/// Handler for GET /api/v1/vendors/{id}
///
/// Gets detailed information about a specific vendor.
///
/// # Path Parameters
///
/// - `id`: Vendor UUID
///
/// # Response
///
/// - 200 OK: Vendor details
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks vendors:read permission
/// - 404 Not Found: Vendor doesn't exist
pub async fn get_vendor_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<VendorResponse>, Response> {
    require_permission(&ctx, "vendors:read")?;

    let use_case = GetVendorUseCase::new(state.vendor_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Update Vendor Handler
// =============================================================================

/// Handler for PUT /api/v1/vendors/{id}
///
/// Updates an existing vendor's details.
///
/// # Path Parameters
///
/// - `id`: Vendor UUID
///
/// # Request Body
///
/// All fields are optional - only provided fields will be updated.
///
/// ```json
/// {
///   "name": "New Name",
///   "legal_name": "New Legal Name",
///   "tax_id": "NewRTN123",
///   "email": "newemail@example.com",
///   "phone": "+504 9999-9999",
///   "address": "New Address",
///   "payment_terms_days": 45,
///   "currency": "USD",
///   "notes": "Updated notes"
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Vendor successfully updated
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks vendors:update permission
/// - 404 Not Found: Vendor doesn't exist
/// - 409 Conflict: Duplicate tax ID
pub async fn update_vendor_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<UpdateVendorCommand>,
) -> Result<Json<VendorResponse>, Response> {
    require_permission(&ctx, "vendors:update")?;

    let use_case = UpdateVendorUseCase::new(state.vendor_repo());

    let response = use_case
        .execute(id, command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Activate Vendor Handler
// =============================================================================

/// Handler for PUT /api/v1/vendors/{id}/activate
///
/// Activates a deactivated vendor.
///
/// # Path Parameters
///
/// - `id`: Vendor UUID
///
/// # Response
///
/// - 200 OK: Vendor successfully activated
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks vendors:update permission
/// - 404 Not Found: Vendor doesn't exist
pub async fn activate_vendor_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<VendorResponse>, Response> {
    require_permission(&ctx, "vendors:update")?;

    let use_case = ToggleVendorStatusUseCase::new(state.vendor_repo());

    let response = use_case
        .activate(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Deactivate Vendor Handler
// =============================================================================

/// Handler for PUT /api/v1/vendors/{id}/deactivate
///
/// Deactivates an active vendor.
///
/// # Path Parameters
///
/// - `id`: Vendor UUID
///
/// # Response
///
/// - 200 OK: Vendor successfully deactivated
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks vendors:update permission
/// - 404 Not Found: Vendor doesn't exist
pub async fn deactivate_vendor_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<VendorResponse>, Response> {
    require_permission(&ctx, "vendors:update")?;

    let use_case = ToggleVendorStatusUseCase::new(state.vendor_repo());

    let response = use_case
        .deactivate(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
