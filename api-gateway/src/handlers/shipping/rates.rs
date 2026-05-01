// ShippingRate handlers + Calculate.

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
use shipping::{
    CalculateShippingCommand, CalculateShippingUseCase, CreateShippingRateCommand,
    CreateShippingRateUseCase, DeleteShippingRateUseCase, ShippingOptionsResponse,
    ShippingRateResponse, UpdateShippingRateCommand, UpdateShippingRateUseCase,
};

pub async fn create_shipping_rate_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<CreateShippingRateCommand>,
) -> Result<(StatusCode, Json<ShippingRateResponse>), Response> {
    require_permission(&ctx, "shipping:create")?;
    let uc = CreateShippingRateUseCase::new(state.shipping_rate_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn update_shipping_rate_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<UpdateShippingRateCommand>,
) -> Result<Json<ShippingRateResponse>, Response> {
    require_permission(&ctx, "shipping:update")?;
    cmd.rate_id = id;
    let uc = UpdateShippingRateUseCase::new(state.shipping_rate_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn delete_shipping_rate_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "shipping:delete")?;
    let uc = DeleteShippingRateUseCase::new(state.shipping_rate_repo());
    uc.execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(StatusCode::NO_CONTENT)
}

/// Public calculate (still behind auth — checkout requires session).
pub async fn calculate_shipping_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<CalculateShippingCommand>,
) -> Result<Json<ShippingOptionsResponse>, Response> {
    require_permission(&ctx, "shipping:read")?;
    let uc = CalculateShippingUseCase::new(
        state.shipping_method_repo(),
        state.shipping_zone_repo(),
        state.shipping_rate_repo(),
    );
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}
