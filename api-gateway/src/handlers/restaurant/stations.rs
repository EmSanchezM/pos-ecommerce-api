use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use restaurant_operations::{
    CreateKitchenStationCommand, CreateKitchenStationUseCase, DeactivateKitchenStationUseCase,
    KitchenStationId, KitchenStationResponse, ListKitchenStationsUseCase,
    UpdateKitchenStationCommand, UpdateKitchenStationUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListStationsQuery {
    pub store_id: Uuid,
    pub include_inactive: Option<bool>,
}

pub async fn list_stations_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListStationsQuery>,
) -> Result<Json<Vec<KitchenStationResponse>>, Response> {
    require_permission(&ctx, "restaurant:read_station")?;
    let only_active = !params.include_inactive.unwrap_or(false);
    let use_case = ListKitchenStationsUseCase::new(state.kitchen_station_repo());
    let stations = use_case
        .execute(params.store_id, only_active)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        stations.iter().map(KitchenStationResponse::from).collect(),
    ))
}

pub async fn create_station_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateKitchenStationCommand>,
) -> Result<Json<KitchenStationResponse>, Response> {
    require_permission(&ctx, "restaurant:write_station")?;
    let use_case = CreateKitchenStationUseCase::new(state.kitchen_station_repo());
    let station = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(KitchenStationResponse::from(&station)))
}

pub async fn update_station_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateKitchenStationCommand>,
) -> Result<Json<KitchenStationResponse>, Response> {
    require_permission(&ctx, "restaurant:write_station")?;
    let use_case = UpdateKitchenStationUseCase::new(state.kitchen_station_repo());
    let station = use_case
        .execute(KitchenStationId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(KitchenStationResponse::from(&station)))
}

pub async fn deactivate_station_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, Response> {
    require_permission(&ctx, "restaurant:write_station")?;
    let use_case = DeactivateKitchenStationUseCase::new(state.kitchen_station_repo());
    use_case
        .execute(KitchenStationId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
