// Recipe HTTP handlers for the API Gateway
//
// These handlers implement the REST endpoints for recipe management:
// - POST /api/recipes - Create a new recipe
// - GET /api/recipes - List recipes with pagination
// - GET /api/recipes/{id} - Get recipe details with ingredients
// - GET /api/products/{product_id}/recipe - Get active recipe for product
// - PUT /api/recipes/{id} - Update recipe
// - POST /api/recipes/{recipe_id}/calculate-cost - Calculate recipe cost

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use inventory::{
    CalculateRecipeCostUseCase, CreateRecipeCommand, CreateRecipeUseCase, GetProductRecipeUseCase,
    GetRecipeUseCase, ListRecipesQuery, ListRecipesUseCase, PaginatedResponse, RecipeCostResult,
    RecipeDetailResponse, RecipeResponse, UpdateRecipeCommand, UpdateRecipeUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

use super::products::{default_page, default_page_size};

// =============================================================================
// Query DTOs
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

// =============================================================================
// Create Recipe Handler
// =============================================================================

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

// =============================================================================
// List Recipes Handler
// =============================================================================

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

// =============================================================================
// Get Recipe Handler
// =============================================================================

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

// =============================================================================
// Get Product Recipe Handler
// =============================================================================

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

// =============================================================================
// Update Recipe Handler
// =============================================================================

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

// =============================================================================
// Calculate Recipe Cost Handler
// =============================================================================

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
