// Listing CRUD handlers (auth required).

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use catalog::{
    CreateListingCommand, CreateListingUseCase, DeleteListingUseCase, GetListingDetailUseCase,
    GetListingUseCase, ListingDetailResponse, ListingResponse, PublishListingUseCase,
    UnpublishListingUseCase, UpdateListingCommand, UpdateListingUseCase,
};

pub async fn create_listing_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<CreateListingCommand>,
) -> Result<(StatusCode, Json<ListingResponse>), Response> {
    require_permission(&ctx, "catalog:create")?;
    let uc = CreateListingUseCase::new(state.listing_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn update_listing_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<UpdateListingCommand>,
) -> Result<Json<ListingResponse>, Response> {
    require_permission(&ctx, "catalog:update")?;
    cmd.listing_id = id;
    let uc = UpdateListingUseCase::new(state.listing_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn publish_listing_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ListingResponse>, Response> {
    require_permission(&ctx, "catalog:update")?;
    let uc = PublishListingUseCase::new(state.listing_repo());
    let resp = uc
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn unpublish_listing_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ListingResponse>, Response> {
    require_permission(&ctx, "catalog:update")?;
    let uc = UnpublishListingUseCase::new(state.listing_repo());
    let resp = uc
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn delete_listing_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "catalog:delete")?;
    let uc = DeleteListingUseCase::new(state.listing_repo());
    uc.execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(StatusCode::NO_CONTENT)
}

/// Authenticated detail (manager view, includes unpublished).
pub async fn get_listing_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ListingResponse>, Response> {
    require_permission(&ctx, "catalog:read")?;
    let uc = GetListingUseCase::new(state.listing_repo());
    let resp = uc
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

/// Authenticated detail with images and rating.
pub async fn get_listing_detail_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ListingDetailResponse>, Response> {
    require_permission(&ctx, "catalog:read")?;
    let uc = GetListingDetailUseCase::new(
        state.listing_repo(),
        state.catalog_image_repo(),
        state.review_repo(),
    );
    let resp = uc
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

#[derive(Debug, serde::Deserialize)]
pub struct ListingsListQuery {
    pub store_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub is_published: Option<bool>,
    pub is_featured: Option<bool>,
    pub search: Option<String>,
    pub min_price: Option<rust_decimal::Decimal>,
    pub max_price: Option<rust_decimal::Decimal>,
    pub sort_by: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_listings_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<ListingsListQuery>,
) -> Result<Json<catalog::ListingListResponse>, Response> {
    require_permission(&ctx, "catalog:read")?;
    let uc = catalog::SearchListingsUseCase::new(state.listing_repo());
    let resp = uc
        .execute(catalog::SearchListingsQuery {
            store_id: q.store_id,
            category_id: q.category_id,
            is_published: q.is_published,
            is_featured: q.is_featured,
            search: q.search,
            min_price: q.min_price,
            max_price: q.max_price,
            sort_by: q.sort_by,
            page: q.page,
            page_size: q.page_size,
        })
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}
