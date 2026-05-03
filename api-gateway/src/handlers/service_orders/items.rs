use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use service_orders::{
    AddItemCommand, AddItemUseCase, RemoveItemUseCase, ServiceOrderId, ServiceOrderItemId,
    ServiceOrderItemResponse, UpdateItemCommand, UpdateItemUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::require_feature;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

pub async fn add_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(order_id): Path<Uuid>,
    Json(cmd): Json<AddItemCommand>,
) -> Result<Json<ServiceOrderItemResponse>, Response> {
    require_permission(&ctx, "service_orders:write_item")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    let use_case = AddItemUseCase::new(state.service_order_repo(), state.service_order_item_repo());
    let item = use_case
        .execute(ServiceOrderId::from_uuid(order_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ServiceOrderItemResponse::from(&item)))
}

pub async fn update_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((_order_id, item_id)): Path<(Uuid, Uuid)>,
    Json(cmd): Json<UpdateItemCommand>,
) -> Result<Json<ServiceOrderItemResponse>, Response> {
    require_permission(&ctx, "service_orders:write_item")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    let use_case =
        UpdateItemUseCase::new(state.service_order_repo(), state.service_order_item_repo());
    let item = use_case
        .execute(ServiceOrderItemId::from_uuid(item_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ServiceOrderItemResponse::from(&item)))
}

pub async fn remove_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((_order_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, Response> {
    require_permission(&ctx, "service_orders:write_item")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    let use_case =
        RemoveItemUseCase::new(state.service_order_repo(), state.service_order_item_repo());
    use_case
        .execute(ServiceOrderItemId::from_uuid(item_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
