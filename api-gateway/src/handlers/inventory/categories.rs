// Category Handlers
//
// REST endpoints for product categories:
// - POST /api/v1/categories - Create a category
// - GET /api/v1/categories - List categories (flat or tree)
// - GET /api/v1/categories/{id} - Get category details
// - GET /api/v1/categories/{id}/children - Get child categories
// - PUT /api/v1/categories/{id} - Update a category
// - DELETE /api/v1/categories/{id} - Delete (deactivate) a category

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use inventory::{
    CategoryResponse, CategoryTreeResponse, CreateCategoryCommand, CreateCategoryUseCase,
    DeleteCategoryUseCase, GetCategoryUseCase, ListCategoriesUseCase, UpdateCategoryCommand,
    UpdateCategoryUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

/// Query parameters for listing categories
#[derive(Debug, Deserialize)]
pub struct ListCategoriesQueryParams {
    /// Format: "flat" (default) or "tree"
    #[serde(default = "default_format")]
    pub format: String,
    /// Optional parent_id to list children of a specific category
    pub parent_id: Option<Uuid>,
}

fn default_format() -> String {
    "flat".to_string()
}

/// Handler for POST /api/v1/categories
pub async fn create_category_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreateCategoryCommand>,
) -> Result<(StatusCode, Json<CategoryResponse>), Response> {
    require_permission(&ctx, "categories:create")?;

    let use_case = CreateCategoryUseCase::new(state.category_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Handler for GET /api/v1/categories
pub async fn list_categories_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListCategoriesQueryParams>,
) -> Result<Response, Response> {
    require_permission(&ctx, "categories:read")?;

    let use_case = ListCategoriesUseCase::new(state.category_repo());

    if let Some(parent_id) = params.parent_id {
        let response = use_case
            .execute_children(parent_id)
            .await
            .map_err(|e| AppError::from(e).into_response())?;
        Ok(Json(response).into_response())
    } else if params.format == "tree" {
        let response: Vec<CategoryTreeResponse> = use_case
            .execute_tree()
            .await
            .map_err(|e| AppError::from(e).into_response())?;
        Ok(Json(response).into_response())
    } else {
        let response = use_case
            .execute_flat()
            .await
            .map_err(|e| AppError::from(e).into_response())?;
        Ok(Json(response).into_response())
    }
}

/// Handler for GET /api/v1/categories/{id}
pub async fn get_category_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CategoryResponse>, Response> {
    require_permission(&ctx, "categories:read")?;

    let use_case = GetCategoryUseCase::new(state.category_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for GET /api/v1/categories/{id}/children
pub async fn get_category_children_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<CategoryResponse>>, Response> {
    require_permission(&ctx, "categories:read")?;

    let use_case = ListCategoriesUseCase::new(state.category_repo());

    let response = use_case
        .execute_children(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/v1/categories/{id}
pub async fn update_category_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<UpdateCategoryCommand>,
) -> Result<Json<CategoryResponse>, Response> {
    require_permission(&ctx, "categories:update")?;

    let use_case = UpdateCategoryUseCase::new(state.category_repo());

    let response = use_case
        .execute(id, command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for DELETE /api/v1/categories/{id}
pub async fn delete_category_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "categories:delete")?;

    let use_case = DeleteCategoryUseCase::new(state.category_repo());

    use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(StatusCode::NO_CONTENT)
}
