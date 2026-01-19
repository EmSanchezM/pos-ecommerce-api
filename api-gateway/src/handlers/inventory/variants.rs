// Variant HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for product variant management:
// - POST /api/products/{product_id}/variants - Create a new variant
// - GET /api/products/{product_id}/variants - List variants
// - GET /api/products/{product_id}/variants/{variant_id} - Get variant details
// - PUT /api/products/{product_id}/variants/{variant_id} - Update variant
// - DELETE /api/products/{product_id}/variants/{variant_id} - Soft delete variant

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use inventory::{
    CreateVariantCommand, CreateVariantUseCase, DeleteVariantUseCase, GetVariantUseCase,
    ListVariantsUseCase, UpdateVariantCommand, UpdateVariantUseCase, VariantResponse,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

// =============================================================================
// Create Variant Handler
// =============================================================================

/// Handler for POST /api/products/{product_id}/variants
///
/// Creates a new variant for a product.
///
/// # Path Parameters
///
/// - `product_id`: Parent product UUID
///
/// # Request Body
///
/// ```json
/// {
///   "name": "Red - Large",
///   "variant_attributes": { "color": "red", "size": "L" },
///   "price": 34.99,
///   "cost_price": 15.00,
///   "barcode": "1234567890123"
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Variant successfully created
/// - 400 Bad Request: Validation error or variants not enabled
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks products:create permission
/// - 404 Not Found: Product doesn't exist
/// - 409 Conflict: Duplicate barcode
pub async fn create_variant_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(product_id): Path<Uuid>,
    Json(mut command): Json<CreateVariantCommand>,
) -> Result<(StatusCode, Json<VariantResponse>), Response> {
    require_permission(&ctx, "products:create")?;

    // Set the product_id from the path parameter
    command.product_id = product_id;

    let use_case = CreateVariantUseCase::new(state.product_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

// =============================================================================
// List Variants Handler
// =============================================================================

/// Handler for GET /api/products/{product_id}/variants
///
/// Lists all variants for a product.
///
/// # Path Parameters
///
/// - `product_id`: Parent product UUID
///
/// # Response
///
/// - 200 OK: List of variants
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Product doesn't exist
pub async fn list_variants_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<VariantResponse>>, Response> {
    let use_case = ListVariantsUseCase::new(state.product_repo());

    let response = use_case
        .execute(product_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Get Variant Handler
// =============================================================================

/// Handler for GET /api/products/{product_id}/variants/{variant_id}
///
/// Gets details of a specific variant.
///
/// # Path Parameters
///
/// - `product_id`: Parent product UUID
/// - `variant_id`: Variant UUID
///
/// # Response
///
/// - 200 OK: Variant details
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Product or variant doesn't exist
pub async fn get_variant_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path((product_id, variant_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<VariantResponse>, Response> {
    let use_case = GetVariantUseCase::new(state.product_repo());

    let response = use_case
        .execute(product_id, variant_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Update Variant Handler
// =============================================================================

/// Handler for PUT /api/products/{product_id}/variants/{variant_id}
///
/// Updates an existing variant.
///
/// # Path Parameters
///
/// - `product_id`: Parent product UUID
/// - `variant_id`: Variant UUID
///
/// # Request Body
///
/// All fields are optional - only provided fields will be updated.
///
/// ```json
/// {
///   "name": "Blue - Medium",
///   "variant_attributes": { "color": "blue", "size": "M" },
///   "price": 39.99,
///   "is_active": true
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Variant successfully updated
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks products:update permission
/// - 404 Not Found: Product or variant doesn't exist
/// - 409 Conflict: Duplicate barcode
pub async fn update_variant_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((product_id, variant_id)): Path<(Uuid, Uuid)>,
    Json(command): Json<UpdateVariantCommand>,
) -> Result<Json<VariantResponse>, Response> {
    require_permission(&ctx, "products:update")?;

    let use_case = UpdateVariantUseCase::new(state.product_repo());

    let response = use_case
        .execute(product_id, variant_id, command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// Delete Variant Handler
// =============================================================================

/// Handler for DELETE /api/products/{product_id}/variants/{variant_id}
///
/// Soft deletes a variant by setting is_active to false.
///
/// # Path Parameters
///
/// - `product_id`: Parent product UUID
/// - `variant_id`: Variant UUID
///
/// # Response
///
/// - 204 No Content: Variant successfully deleted
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks products:delete permission
/// - 404 Not Found: Product or variant doesn't exist
pub async fn delete_variant_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((product_id, variant_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "products:delete")?;

    let use_case = DeleteVariantUseCase::new(state.product_repo());

    use_case
        .execute(product_id, variant_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(StatusCode::NO_CONTENT)
}
