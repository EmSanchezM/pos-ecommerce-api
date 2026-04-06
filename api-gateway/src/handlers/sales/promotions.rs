// Promotion Handlers
//
// REST endpoints for promotion management:
// - POST /api/v1/promotions - Create promotion
// - GET /api/v1/promotions - List promotions
// - GET /api/v1/promotions/{id} - Get promotion details
// - PUT /api/v1/promotions/{id} - Update promotion
// - POST /api/v1/promotions/{id}/deactivate - Deactivate promotion
// - POST /api/v1/sales/{sale_id}/apply-promotion - Apply promotion to sale

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use sales::{
    ApplyPromotionUseCase, CreatePromotionCommand, CreatePromotionUseCase,
    DeactivatePromotionUseCase, GetPromotionUseCase, ListPromotionsQuery, ListPromotionsUseCase,
    PromotionResponse, SaleDetailResponse, UpdatePromotionCommand, UpdatePromotionUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

#[derive(Debug, Deserialize)]
pub struct ListPromotionsQueryParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    pub status: Option<String>,
    pub store_id: Option<Uuid>,
    pub search: Option<String>,
}

use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedPromotionResponse {
    pub items: Vec<PromotionResponse>,
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
    pub total_pages: i64,
}

/// Handler for POST /api/v1/promotions
pub async fn create_promotion_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreatePromotionCommand>,
) -> Result<(StatusCode, Json<PromotionResponse>), Response> {
    require_permission(&ctx, "promotions:create")?;

    let use_case = CreatePromotionUseCase::new(state.promotion_repo());

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Handler for GET /api/v1/promotions
pub async fn list_promotions_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListPromotionsQueryParams>,
) -> Result<Json<PaginatedPromotionResponse>, Response> {
    require_permission(&ctx, "promotions:read")?;

    let use_case = ListPromotionsUseCase::new(state.promotion_repo());

    let query = ListPromotionsQuery {
        status: params.status,
        store_id: params.store_id,
        search: params.search,
        page: params.page,
        page_size: params.page_size,
    };

    let (items, total_items) = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    let total_pages = if total_items == 0 {
        1
    } else {
        (total_items + params.page_size - 1) / params.page_size
    };

    Ok(Json(PaginatedPromotionResponse {
        items,
        page: params.page,
        page_size: params.page_size,
        total_items,
        total_pages,
    }))
}

/// Handler for GET /api/v1/promotions/{id}
pub async fn get_promotion_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PromotionResponse>, Response> {
    require_permission(&ctx, "promotions:read")?;

    let use_case = GetPromotionUseCase::new(state.promotion_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/v1/promotions/{id}
pub async fn update_promotion_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<UpdatePromotionCommand>,
) -> Result<Json<PromotionResponse>, Response> {
    require_permission(&ctx, "promotions:update")?;

    let use_case = UpdatePromotionUseCase::new(state.promotion_repo());

    let response = use_case
        .execute(id, command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for POST /api/v1/promotions/{id}/deactivate
pub async fn deactivate_promotion_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PromotionResponse>, Response> {
    require_permission(&ctx, "promotions:update")?;

    let use_case = DeactivatePromotionUseCase::new(state.promotion_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Body for apply-promotion endpoint
#[derive(Debug, Deserialize)]
pub struct ApplyPromotionBody {
    pub promotion_code: String,
}

/// Handler for POST /api/v1/sales/{sale_id}/apply-promotion
pub async fn apply_promotion_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(sale_id): Path<Uuid>,
    Json(body): Json<ApplyPromotionBody>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "promotions:apply")?;

    let use_case = ApplyPromotionUseCase::new(state.promotion_repo(), state.sale_repo());

    let response = use_case
        .execute(sale_id, body.promotion_code)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
