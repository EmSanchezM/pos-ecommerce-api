// ShippingZone handlers.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use super::methods::StoreScopedQuery;
use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use shipping::{
    CreateShippingZoneCommand, CreateShippingZoneUseCase, DeleteShippingZoneUseCase,
    ListShippingZonesUseCase, ShippingZoneResponse, UpdateShippingZoneCommand,
    UpdateShippingZoneUseCase,
};

pub async fn create_shipping_zone_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<CreateShippingZoneCommand>,
) -> Result<(StatusCode, Json<ShippingZoneResponse>), Response> {
    require_permission(&ctx, "shipping:create")?;
    let uc = CreateShippingZoneUseCase::new(state.shipping_zone_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn list_shipping_zones_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<StoreScopedQuery>,
) -> Result<Json<Vec<ShippingZoneResponse>>, Response> {
    require_permission(&ctx, "shipping:read")?;
    let uc = ListShippingZonesUseCase::new(state.shipping_zone_repo());
    let resp = uc
        .execute(q.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn update_shipping_zone_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<UpdateShippingZoneCommand>,
) -> Result<Json<ShippingZoneResponse>, Response> {
    require_permission(&ctx, "shipping:update")?;
    cmd.zone_id = id;
    let uc = UpdateShippingZoneUseCase::new(state.shipping_zone_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn delete_shipping_zone_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "shipping:delete")?;
    let uc = DeleteShippingZoneUseCase::new(state.shipping_zone_repo());
    uc.execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(StatusCode::NO_CONTENT)
}
