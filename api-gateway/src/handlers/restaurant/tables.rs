use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use restaurant_operations::{
    CreateRestaurantTableCommand, CreateRestaurantTableUseCase, DeactivateRestaurantTableUseCase,
    ListRestaurantTablesUseCase, RestaurantTableId, RestaurantTableResponse, SetTableStatusCommand,
    SetTableStatusUseCase, UpdateRestaurantTableCommand, UpdateRestaurantTableUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListTablesQuery {
    pub store_id: Uuid,
    pub include_inactive: Option<bool>,
}

pub async fn list_tables_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListTablesQuery>,
) -> Result<Json<Vec<RestaurantTableResponse>>, Response> {
    require_permission(&ctx, "restaurant:read_table")?;
    let only_active = !params.include_inactive.unwrap_or(false);
    let use_case = ListRestaurantTablesUseCase::new(state.restaurant_table_repo());
    let tables = use_case
        .execute(params.store_id, only_active)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        tables.iter().map(RestaurantTableResponse::from).collect(),
    ))
}

pub async fn create_table_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateRestaurantTableCommand>,
) -> Result<Json<RestaurantTableResponse>, Response> {
    require_permission(&ctx, "restaurant:write_table")?;
    let use_case = CreateRestaurantTableUseCase::new(state.restaurant_table_repo());
    let table = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(RestaurantTableResponse::from(&table)))
}

pub async fn update_table_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateRestaurantTableCommand>,
) -> Result<Json<RestaurantTableResponse>, Response> {
    require_permission(&ctx, "restaurant:write_table")?;
    let use_case = UpdateRestaurantTableUseCase::new(state.restaurant_table_repo());
    let table = use_case
        .execute(RestaurantTableId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(RestaurantTableResponse::from(&table)))
}

pub async fn set_table_status_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<SetTableStatusCommand>,
) -> Result<Json<RestaurantTableResponse>, Response> {
    require_permission(&ctx, "restaurant:write_table")?;
    let use_case = SetTableStatusUseCase::new(state.restaurant_table_repo());
    let table = use_case
        .execute(RestaurantTableId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(RestaurantTableResponse::from(&table)))
}

pub async fn deactivate_table_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, Response> {
    require_permission(&ctx, "restaurant:write_table")?;
    let use_case = DeactivateRestaurantTableUseCase::new(state.restaurant_table_repo());
    use_case
        .execute(RestaurantTableId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
