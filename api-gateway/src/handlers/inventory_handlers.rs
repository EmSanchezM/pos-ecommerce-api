// Inventory HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for inventory management:
// - POST /api/products - Create a new product
// - GET /api/products - List products with pagination
// - GET /api/products/{id} - Get product details
// - PUT /api/products/{id} - Update product
// - DELETE /api/products/{id} - Soft delete product
// - POST /api/recipes - Create a new recipe
// - GET /api/recipes - List recipes with pagination
// - GET /api/recipes/{id} - Get recipe details with ingredients
// - GET /api/products/{product_id}/recipe - Get active recipe for product
// - PUT /api/recipes/{id} - Update recipe
// - POST /api/recipes/{recipe_id}/calculate-cost - Calculate recipe cost
// - GET /api/inventory/stock - List stock with pagination
// - GET /api/inventory/stock/{stock_id} - Get stock details
// - GET /api/stores/{store_id}/inventory - Get store inventory
// - GET /api/products/{product_id}/stock - Get product stock across stores

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use inventory::{
    CalculateRecipeCostUseCase, CreateProductCommand, CreateProductUseCase, CreateRecipeCommand,
    CreateRecipeUseCase, CreateVariantCommand, CreateVariantUseCase, DeleteProductUseCase,
    DeleteVariantUseCase, GetProductRecipeUseCase, GetProductStockUseCase, GetProductUseCase,
    GetRecipeUseCase, GetStockUseCase, GetStoreInventoryUseCase, GetVariantUseCase,
    ListProductsQuery, ListProductsUseCase, ListRecipesQuery, ListRecipesUseCase,
    ListStockQuery, ListStockUseCase, ListVariantsUseCase, PaginatedResponse,
    ProductDetailResponse, ProductResponse, RecipeCostResult, RecipeDetailResponse,
    RecipeResponse, StockDetailResponse, StockResponse, UpdateProductCommand,
    UpdateProductUseCase, UpdateRecipeCommand, UpdateRecipeUseCase, UpdateVariantCommand,
    UpdateVariantUseCase, VariantResponse,
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

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
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

// =============================================================================
// Variant Handlers
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

// =============================================================================
// Recipe Handlers
// =============================================================================

/// Query parameters for listing recipes (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct ListRecipesQueryParams {
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Search term for name/description
    pub search: Option<String>,
}

impl From<ListRecipesQueryParams> for ListRecipesQuery {
    fn from(params: ListRecipesQueryParams) -> Self {
        ListRecipesQuery {
            is_active: params.is_active,
            search: params.search,
            page: params.page,
            page_size: params.page_size,
        }
    }
}

/// Response for recipe cost calculation (serializable)
#[derive(Debug, Clone, Serialize)]
pub struct RecipeCostResponse {
    pub recipe_id: Uuid,
    pub total_ingredient_cost: rust_decimal::Decimal,
    pub cost_per_unit: rust_decimal::Decimal,
    pub yield_quantity: rust_decimal::Decimal,
    pub ingredient_count: usize,
    pub ingredients_with_cost: usize,
}

impl From<RecipeCostResult> for RecipeCostResponse {
    fn from(result: RecipeCostResult) -> Self {
        Self {
            recipe_id: result.recipe_id,
            total_ingredient_cost: result.total_ingredient_cost,
            cost_per_unit: result.cost_per_unit,
            yield_quantity: result.yield_quantity,
            ingredient_count: result.ingredient_count,
            ingredients_with_cost: result.ingredients_with_cost,
        }
    }
}

/// Handler for POST /api/recipes
///
/// Creates a new recipe with ingredients.
///
/// # Request Body
///
/// ```json
/// {
///   "product_id": "uuid",
///   "name": "Recipe Name",
///   "yield_quantity": 10,
///   "ingredients": [
///     {
///       "ingredient_product_id": "uuid",
///       "quantity": 2,
///       "unit_of_measure": "kg",
///       "can_substitute": true,
///       "substitutes": []
///     }
///   ]
/// }
/// ```
///
/// # Response
///
/// - 201 Created: Recipe successfully created
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks recipes:create permission
/// - 409 Conflict: Active recipe already exists for product/variant
pub async fn create_recipe_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreateRecipeCommand>,
) -> Result<(StatusCode, Json<RecipeResponse>), Response> {
    require_permission(&ctx, "recipes:create")?;

    let use_case = CreateRecipeUseCase::new(state.recipe_repo(), state.product_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Handler for GET /api/recipes
///
/// Lists recipes with pagination and optional filters.
///
/// # Query Parameters
///
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `is_active` (optional): Filter by active status
/// - `search` (optional): Search in name/description
///
/// # Response
///
/// - 200 OK: Paginated list of recipes
/// - 401 Unauthorized: Missing or invalid token
pub async fn list_recipes_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Query(params): Query<ListRecipesQueryParams>,
) -> Result<Json<PaginatedResponse<RecipeResponse>>, Response> {
    let use_case = ListRecipesUseCase::new(state.recipe_repo());

    let query: ListRecipesQuery = params.into();
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for GET /api/recipes/{id}
///
/// Gets detailed information about a specific recipe, including ingredients.
///
/// # Path Parameters
///
/// - `id`: Recipe UUID
///
/// # Response
///
/// - 200 OK: Recipe details with ingredients and substitutes
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Recipe doesn't exist
pub async fn get_recipe_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<RecipeDetailResponse>, Response> {
    let use_case = GetRecipeUseCase::new(state.recipe_repo(), state.product_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for GET /api/products/{product_id}/recipe
///
/// Gets the active recipe for a specific product.
///
/// # Path Parameters
///
/// - `product_id`: Product UUID
///
/// # Response
///
/// - 200 OK: Recipe details with ingredients and substitutes
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Product doesn't exist or has no active recipe
pub async fn get_product_recipe_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(product_id): Path<Uuid>,
) -> Result<Json<RecipeDetailResponse>, Response> {
    let use_case = GetProductRecipeUseCase::new(state.recipe_repo(), state.product_repo());

    let response = use_case
        .execute(product_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/recipes/{id}
///
/// Updates an existing recipe's details.
///
/// # Path Parameters
///
/// - `id`: Recipe UUID
///
/// # Request Body
///
/// All fields are optional - only provided fields will be updated.
///
/// ```json
/// {
///   "name": "New Name",
///   "yield_quantity": 15,
///   "is_active": true
/// }
/// ```
///
/// # Response
///
/// - 200 OK: Recipe successfully updated
/// - 400 Bad Request: Validation error
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks recipes:update permission
/// - 404 Not Found: Recipe doesn't exist
pub async fn update_recipe_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<UpdateRecipeCommand>,
) -> Result<Json<RecipeResponse>, Response> {
    require_permission(&ctx, "recipes:update")?;

    let use_case = UpdateRecipeUseCase::new(state.recipe_repo());

    let response = use_case
        .execute(id, command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for POST /api/recipes/{recipe_id}/calculate-cost
///
/// Calculates the cost breakdown for a recipe based on current ingredient costs.
///
/// # Path Parameters
///
/// - `recipe_id`: Recipe UUID
///
/// # Response
///
/// - 200 OK: Recipe cost breakdown
/// - 401 Unauthorized: Missing or invalid token
/// - 404 Not Found: Recipe doesn't exist
pub async fn calculate_recipe_cost_handler(
    State(state): State<AppState>,
    CurrentUser(_ctx): CurrentUser,
    Path(recipe_id): Path<Uuid>,
) -> Result<Json<RecipeCostResponse>, Response> {
    let use_case = CalculateRecipeCostUseCase::new(state.recipe_repo());

    let result = use_case
        .execute(recipe_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(RecipeCostResponse::from(result)))
}

// =============================================================================
// Stock Handlers
// =============================================================================

/// Query parameters for listing stock (HTTP API layer)
#[derive(Debug, Deserialize)]
pub struct ListStockQueryParams {
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Filter by product ID
    pub product_id: Option<Uuid>,
    /// Filter to only show low stock items
    #[serde(default)]
    pub low_stock: bool,
}

impl From<ListStockQueryParams> for ListStockQuery {
    fn from(params: ListStockQueryParams) -> Self {
        ListStockQuery {
            store_id: params.store_id,
            product_id: params.product_id,
            low_stock: params.low_stock,
            page: params.page,
            page_size: params.page_size,
        }
    }
}

/// Handler for GET /api/inventory/stock
///
/// Lists stock records with pagination and optional filters.
///
/// # Query Parameters
///
/// - `page` (optional): Page number (1-based, default: 1)
/// - `page_size` (optional): Items per page (default: 20, max: 100)
/// - `store_id` (optional): Filter by store
/// - `product_id` (optional): Filter by product
/// - `low_stock` (optional): Only show low stock items (default: false)
///
/// # Response
///
/// - 200 OK: Paginated list of stock records
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
pub async fn list_stock_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListStockQueryParams>,
) -> Result<Json<PaginatedResponse<StockResponse>>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = ListStockUseCase::new(state.stock_repo());

    let query: ListStockQuery = params.into();
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for GET /api/inventory/stock/{stock_id}
///
/// Gets detailed information about a specific stock record.
///
/// # Path Parameters
///
/// - `stock_id`: Stock record UUID
///
/// # Response
///
/// - 200 OK: Stock details with product/variant info
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
/// - 404 Not Found: Stock record doesn't exist
pub async fn get_stock_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(stock_id): Path<Uuid>,
) -> Result<Json<StockDetailResponse>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = GetStockUseCase::new(state.stock_repo(), state.product_repo());

    let response = use_case
        .execute(stock_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for GET /api/stores/{store_id}/inventory
///
/// Gets all inventory for a specific store.
///
/// # Path Parameters
///
/// - `store_id`: Store UUID
///
/// # Response
///
/// - 200 OK: List of stock records for the store
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
pub async fn get_store_inventory_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(store_id): Path<Uuid>,
) -> Result<Json<Vec<StockResponse>>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = GetStoreInventoryUseCase::new(state.stock_repo());

    let response = use_case
        .execute(store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for GET /api/products/{product_id}/stock
///
/// Gets stock levels for a product across all stores.
///
/// # Path Parameters
///
/// - `product_id`: Product UUID
///
/// # Response
///
/// - 200 OK: List of stock records for the product across all stores
/// - 401 Unauthorized: Missing or invalid token
/// - 403 Forbidden: User lacks inventory:read permission
/// - 404 Not Found: Product doesn't exist
pub async fn get_product_stock_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<StockResponse>>, Response> {
    require_permission(&ctx, "inventory:read")?;

    let use_case = GetProductStockUseCase::new(state.stock_repo(), state.product_repo());

    let response = use_case
        .execute(product_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
