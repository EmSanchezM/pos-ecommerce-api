// Wishlist handlers (customer).

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use catalog::{
    AddToWishlistUseCase, AddWishlistItemCommand, GetWishlistUseCase, RemoveFromWishlistUseCase,
    RemoveWishlistItemCommand, WishlistResponse,
};

#[derive(Debug, Deserialize)]
pub struct WishlistQuery {
    pub customer_id: Uuid,
    pub store_id: Uuid,
}

pub async fn get_wishlist_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<WishlistQuery>,
) -> Result<Json<WishlistResponse>, Response> {
    require_permission(&ctx, "catalog:read")?;
    let uc = GetWishlistUseCase::new(state.wishlist_repo());
    let resp = uc
        .execute(q.customer_id, q.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn add_to_wishlist_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<AddWishlistItemCommand>,
) -> Result<Json<WishlistResponse>, Response> {
    require_permission(&ctx, "catalog:read")?;
    let uc = AddToWishlistUseCase::new(state.wishlist_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn remove_from_wishlist_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<RemoveWishlistItemCommand>,
) -> Result<Json<WishlistResponse>, Response> {
    require_permission(&ctx, "catalog:read")?;
    let uc = RemoveFromWishlistUseCase::new(state.wishlist_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}
