// ShippingMethod handlers.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use shipping::{
    CreateShippingMethodCommand, CreateShippingMethodUseCase, DeleteShippingMethodUseCase,
    ListShippingMethodsUseCase, ShippingMethodResponse, UpdateShippingMethodCommand,
    UpdateShippingMethodUseCase,
};

#[derive(Debug, Deserialize)]
pub struct StoreScopedQuery {
    pub store_id: Uuid,
}

pub async fn create_shipping_method_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<CreateShippingMethodCommand>,
) -> Result<(StatusCode, Json<ShippingMethodResponse>), Response> {
    require_permission(&ctx, "shipping:create")?;
    let uc = CreateShippingMethodUseCase::new(state.shipping_method_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn list_shipping_methods_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<StoreScopedQuery>,
) -> Result<Json<Vec<ShippingMethodResponse>>, Response> {
    require_permission(&ctx, "shipping:read")?;
    let uc = ListShippingMethodsUseCase::new(state.shipping_method_repo());
    let resp = uc
        .execute(q.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn update_shipping_method_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<UpdateShippingMethodCommand>,
) -> Result<Json<ShippingMethodResponse>, Response> {
    require_permission(&ctx, "shipping:update")?;
    cmd.method_id = id;
    let uc = UpdateShippingMethodUseCase::new(state.shipping_method_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn delete_shipping_method_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "shipping:delete")?;
    let uc = DeleteShippingMethodUseCase::new(state.shipping_method_repo());
    uc.execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(StatusCode::NO_CONTENT)
}
