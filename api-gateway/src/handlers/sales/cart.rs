// Cart handlers for the Sales module (E-commerce)

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use sales::{AddCartItemCommand, CartResponse, CreateCartCommand, UpdateCartItemCommand};

pub async fn create_cart_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(command): JsonBody<CreateCartCommand>,
) -> Result<(StatusCode, Json<CartResponse>), Response> {
    require_permission(&ctx, "sales:manage_cart")?;

    let use_case = sales::CreateCartUseCase::new(state.cart_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_cart_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(cart_id): Path<Uuid>,
) -> Result<Json<CartResponse>, Response> {
    require_permission(&ctx, "sales:manage_cart")?;

    let use_case = sales::GetCartUseCase::new(state.cart_repo());

    let response = use_case
        .execute(cart_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn add_cart_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(cart_id): Path<Uuid>,
    JsonBody(command): JsonBody<AddCartItemCommand>,
) -> Result<(StatusCode, Json<CartResponse>), Response> {
    require_permission(&ctx, "sales:manage_cart")?;

    let use_case = sales::AddCartItemUseCase::new(state.cart_repo());

    let mut cmd = command;
    cmd.cart_id = cart_id;

    let response = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_cart_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((cart_id, item_id)): Path<(Uuid, Uuid)>,
    JsonBody(command): JsonBody<UpdateCartItemCommand>,
) -> Result<Json<CartResponse>, Response> {
    require_permission(&ctx, "sales:manage_cart")?;

    let use_case = sales::UpdateCartItemUseCase::new(state.cart_repo());

    let mut cmd = command;
    cmd.cart_id = cart_id;
    cmd.item_id = item_id;

    let response = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn remove_cart_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((cart_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CartResponse>, Response> {
    require_permission(&ctx, "sales:manage_cart")?;

    let use_case = sales::RemoveCartItemUseCase::new(state.cart_repo());

    let response = use_case
        .execute(cart_id, item_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn clear_cart_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(cart_id): Path<Uuid>,
) -> Result<Json<CartResponse>, Response> {
    require_permission(&ctx, "sales:manage_cart")?;

    let use_case = sales::ClearCartUseCase::new(state.cart_repo());

    let response = use_case
        .execute(cart_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
