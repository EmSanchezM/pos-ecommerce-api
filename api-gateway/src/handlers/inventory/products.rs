// Product HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for product management:
// - POST /api/products - Create a new product
// - GET /api/products - List products with pagination
// - GET /api/products/{id} - Get product details
// - PUT /api/products/{id} - Update product
// - DELETE /api/products/{id} - Soft delete product

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use inventory::{
    CreateProductCommand, CreateProductUseCase, DeleteProductUseCase, GetProductUseCase,
    ListProductsQuery, ListProductsUseCase, PaginatedResponse, ProductDetailResponse,
    ProductResponse, UpdateProductCommand, UpdateProductUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

// =============================================================================
// Query DTOs
// =============================================================================

/// Query parameters for listing products (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct ListProductsQueryParams {
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filter by category ID
    pub category_id: Option<Uuid>,
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Search term for name/description
    pub search: Option<String>,
}

pub(crate) fn default_page() -> i64 {
    1
}

pub(crate) fn default_page_size() -> i64 {
    20
}

impl From<ListProductsQueryParams> for ListProductsQuery {
    fn from(params: ListProductsQueryParams) -> Self {
        ListProductsQuery {
            category_id: params.category_id,
            is_active: params.is_active,
            search: params.search,
            page: params.page,
            page_size: params.page_size,
        }
    }
}

// =============================================================================
// Create Product Handler
// =============================================================================

/// Handler for POST /api/products
///
/// Creates a new product with auto-generated SKU.
///
/// # Request Body
///
/// ```json
/// {
///   "name": "Product Name",
///   "unit_of_measure": "unit",
///   "base_price": 99.99,
///   "cost_price": 50.00,
///   "barcode": "1234567890123",
///   "category_id": "uuid",
///   "is_trackable": true
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Product successfully created
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks products:create permission
/// - 409 Conflict: Duplicate barcode
pub async fn create_product_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreateProductCommand>,
) -> Result<(StatusCode, Json<ProductResponse>), Response> {
    require_permission(&ctx, "products:create")?;

    let use_case = CreateProductUseCase::new(
        state.product_repo(),
        state.category_repo(),
        state.audit_repo(),
    );

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

// =============================================================================
// List Products Handler
// =============================================================================

/// Handler for GET /api/products
///
/// Lists products with pagination and optional filters.
///
/// # Query Parameters
///
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `category_id` (optional): Filter by category
/// - `is_active` (optional): Filter by active status
/// - `search` (optional): Search in name/description
///
/// # Response
///
/// - 200 OK: Paginated list of products
/// - 401 Unauthorized: Missing or invalid token
pub async fn list_products_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Query(params): Query<ListProductsQueryParams>,
) -> Result<Json<PaginatedResponse<ProductResponse>>, Response> {
    let use_case = ListProductsUseCase::new(state.product_repo());

    let query: ListProductsQuery = params.into();
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Product Handler
// =============================================================================

/// Handler for GET /api/products/{id}
///
/// Gets detailed information about a specific product, including variants.
///
/// # Path Parameters
///
/// - `id`: Product UUID
///
/// # Response
///
/// - 200 OK: Product details with variants
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Product doesn't exist
pub async fn get_product_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductDetailResponse>, Response> {
    let use_case = GetProductUseCase::new(state.product_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Update Product Handler
// =============================================================================

/// Handler for PUT /api/products/{id}
///
/// Updates an existing product's details.
///
/// # Path Parameters
///
/// - `id`: Product UUID
///
/// # Request Body
///
/// All fields are optional - only provided fields will be updated.
///
/// ```json
/// {
///   "name": "New Name",
///   "base_price": 149.99,
///   "is_active": true
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Product successfully updated
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks products:update permission
/// - 404 Not Found: Product doesn't exist
/// - 409 Conflict: Duplicate barcode
pub async fn update_product_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<UpdateProductCommand>,
) -> Result<Json<ProductResponse>, Response> {
    require_permission(&ctx, "products:update")?;

    let use_case = UpdateProductUseCase::new(
        state.product_repo(),
        state.category_repo(),
        state.audit_repo(),
    );

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(id, command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Delete Product Handler
// =============================================================================

/// Handler for DELETE /api/products/{id}
///
/// Soft deletes a product by setting is_active to false.
///
/// # Path Parameters
///
/// - `id`: Product UUID
///
/// # Response
///
/// - 204 No Content: Product successfully deleted
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks products:delete permission
/// - 404 Not Found: Product doesn't exist
pub async fn delete_product_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "products:delete")?;

    let use_case = DeleteProductUseCase::new(state.product_repo(), state.audit_repo());

    let actor_id = *ctx.user_id();
    use_case
        .execute(id, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(StatusCode::NO_CONTENT)
}
