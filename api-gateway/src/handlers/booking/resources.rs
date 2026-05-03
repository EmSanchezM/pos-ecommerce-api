//! Resource endpoints (people, equipment, rooms) + weekly calendar.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use booking::{
    CreateResourceCommand, CreateResourceUseCase, DeactivateResourceUseCase,
    GetResourceCalendarUseCase, ListResourcesUseCase, ResourceCalendarEntryResponse, ResourceId,
    ResourceResponse, SetResourceCalendarCommand, SetResourceCalendarUseCase,
    UpdateResourceCommand, UpdateResourceUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListResourcesQuery {
    pub store_id: Uuid,
    pub include_inactive: Option<bool>,
}

pub async fn list_resources_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListResourcesQuery>,
) -> Result<Json<Vec<ResourceResponse>>, Response> {
    require_permission(&ctx, "booking:read_resource")?;
    let only_active = !params.include_inactive.unwrap_or(false);
    let use_case = ListResourcesUseCase::new(state.resource_repo());
    let resources = use_case
        .execute(params.store_id, only_active)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resources.iter().map(ResourceResponse::from).collect()))
}

pub async fn create_resource_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateResourceCommand>,
) -> Result<Json<ResourceResponse>, Response> {
    require_permission(&ctx, "booking:write_resource")?;
    let use_case = CreateResourceUseCase::new(state.resource_repo());
    let resource = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ResourceResponse::from(&resource)))
}

pub async fn update_resource_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateResourceCommand>,
) -> Result<Json<ResourceResponse>, Response> {
    require_permission(&ctx, "booking:write_resource")?;
    let use_case = UpdateResourceUseCase::new(state.resource_repo());
    let resource = use_case
        .execute(ResourceId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ResourceResponse::from(&resource)))
}

pub async fn deactivate_resource_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, Response> {
    require_permission(&ctx, "booking:write_resource")?;
    let use_case = DeactivateResourceUseCase::new(state.resource_repo());
    use_case
        .execute(ResourceId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn set_resource_calendar_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<SetResourceCalendarCommand>,
) -> Result<Json<Vec<ResourceCalendarEntryResponse>>, Response> {
    require_permission(&ctx, "booking:write_resource")?;
    let use_case =
        SetResourceCalendarUseCase::new(state.resource_repo(), state.resource_calendar_repo());
    let entries = use_case
        .execute(ResourceId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        entries
            .iter()
            .map(ResourceCalendarEntryResponse::from)
            .collect(),
    ))
}

pub async fn get_resource_calendar_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ResourceCalendarEntryResponse>>, Response> {
    require_permission(&ctx, "booking:read_resource")?;
    let use_case = GetResourceCalendarUseCase::new(state.resource_calendar_repo());
    let entries = use_case
        .execute(ResourceId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        entries
            .iter()
            .map(ResourceCalendarEntryResponse::from)
            .collect(),
    ))
}
