// Payment gateway handlers
//
// AUTHORIZATION
// -------------
// Creating, updating and deleting payment gateways is restricted to
// super admins via `require_super_admin`. Reading and listing only
// require the `payment_gateways:read` permission so store managers can
// see (but not modify) the configured gateways.

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
use crate::middleware::permission::{require_permission, require_super_admin};
use crate::state::AppState;
use payments::{
    ConfigureGatewayCommand, ConfigureGatewayUseCase, DeleteGatewayUseCase, GatewayResponse,
    GetGatewayUseCase, ListGatewaysUseCase, UpdateGatewayCommand, UpdateGatewayUseCase,
};

#[derive(Debug, Deserialize)]
pub struct ListGatewaysQuery {
    pub store_id: Uuid,
}

pub async fn configure_gateway_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(command): JsonBody<ConfigureGatewayCommand>,
) -> Result<(StatusCode, Json<GatewayResponse>), Response> {
    require_super_admin(&ctx)?;

    let use_case = ConfigureGatewayUseCase::new(state.payment_gateway_repo());
    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_gateways_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(query): Query<ListGatewaysQuery>,
) -> Result<Json<Vec<GatewayResponse>>, Response> {
    require_permission(&ctx, "payment_gateways:read")?;

    let use_case = ListGatewaysUseCase::new(state.payment_gateway_repo());
    let response = use_case
        .execute(query.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn get_gateway_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<GatewayResponse>, Response> {
    require_permission(&ctx, "payment_gateways:read")?;

    let use_case = GetGatewayUseCase::new(state.payment_gateway_repo());
    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn update_gateway_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut command): JsonBody<UpdateGatewayCommand>,
) -> Result<Json<GatewayResponse>, Response> {
    require_super_admin(&ctx)?;
    command.gateway_id = id;

    let use_case = UpdateGatewayUseCase::new(state.payment_gateway_repo());
    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn delete_gateway_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_super_admin(&ctx)?;

    let use_case = DeleteGatewayUseCase::new(state.payment_gateway_repo());
    use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(StatusCode::NO_CONTENT)
}
