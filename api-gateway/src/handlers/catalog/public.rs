// Public catalog handlers — NO auth required (SEO + anonymous browsing).
//
// Only published listings/approved reviews are exposed.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;
use catalog::{
    CatalogError, GetFeaturedListingsUseCase, GetListingBySlugUseCase, ListReviewsUseCase,
    ListingDetailResponse, ListingListResponse, ListingResponse, ReviewListResponse,
    SearchListingsQuery, SearchListingsUseCase,
};

#[derive(Debug, Deserialize)]
pub struct PublicSearchQuery {
    pub store_id: Uuid,
    pub category_id: Option<Uuid>,
    pub is_featured: Option<bool>,
    pub search: Option<String>,
    pub min_price: Option<rust_decimal::Decimal>,
    pub max_price: Option<rust_decimal::Decimal>,
    pub sort_by: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn public_search_listings_handler(
    State(state): State<AppState>,
    Query(q): Query<PublicSearchQuery>,
) -> Result<Json<ListingListResponse>, Response> {
    let uc = SearchListingsUseCase::new(state.listing_repo());
    // Public search is forced to only return published listings.
    let resp = uc
        .execute(SearchListingsQuery {
            store_id: Some(q.store_id),
            category_id: q.category_id,
            is_published: Some(true),
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

#[derive(Debug, Deserialize)]
pub struct PublicFeaturedQuery {
    pub store_id: Uuid,
    pub limit: Option<i64>,
}

pub async fn public_featured_handler(
    State(state): State<AppState>,
    Query(q): Query<PublicFeaturedQuery>,
) -> Result<Json<Vec<ListingResponse>>, Response> {
    let uc = GetFeaturedListingsUseCase::new(state.listing_repo());
    let resp = uc
        .execute(q.store_id, q.limit)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

#[derive(Debug, Deserialize)]
pub struct PublicSlugQuery {
    pub store_id: Uuid,
}

pub async fn public_get_by_slug_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<PublicSlugQuery>,
) -> Result<Json<ListingDetailResponse>, Response> {
    let uc = GetListingBySlugUseCase::new(
        state.listing_repo(),
        state.catalog_image_repo(),
        state.review_repo(),
    );
    let detail = uc
        .execute(q.store_id, &slug)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    // Only show if published.
    if !detail.listing.is_published {
        return Err(AppError::from(CatalogError::ListingUnpublished).into_response());
    }
    Ok(Json(detail))
}

#[derive(Debug, Deserialize)]
pub struct PublicReviewsQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn public_listing_reviews_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<PublicReviewsQuery>,
) -> Result<Json<ReviewListResponse>, Response> {
    let uc = ListReviewsUseCase::new(state.review_repo());
    let resp = uc
        .execute(id, q.page, q.page_size)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}
