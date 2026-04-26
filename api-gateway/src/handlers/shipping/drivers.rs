// Driver handlers.

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
    CreateDriverCommand, CreateDriverUseCase, DeleteDriverUseCase, DriverResponse,
    ListDriversUseCase, UpdateDriverCommand, UpdateDriverUseCase,
};

#[derive(Debug, Deserialize)]
pub struct ListDriversQuery {
    pub store_id: Uuid,
    #[serde(default)]
    pub only_available: bool,
}

pub async fn create_driver_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<CreateDriverCommand>,
) -> Result<(StatusCode, Json<DriverResponse>), Response> {
    require_permission(&ctx, "drivers:create")?;
    let uc = CreateDriverUseCase::new(state.driver_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn list_drivers_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<ListDriversQuery>,
) -> Result<Json<Vec<DriverResponse>>, Response> {
    require_permission(&ctx, "drivers:read")?;
    let uc = ListDriversUseCase::new(state.driver_repo());
    let resp = uc
        .execute(q.store_id, q.only_available)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn update_driver_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<UpdateDriverCommand>,
) -> Result<Json<DriverResponse>, Response> {
    require_permission(&ctx, "drivers:update")?;
    cmd.driver_id = id;
    let uc = UpdateDriverUseCase::new(state.driver_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn delete_driver_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "drivers:delete")?;
    let uc = DeleteDriverUseCase::new(state.driver_repo());
    uc.execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(StatusCode::NO_CONTENT)
}
