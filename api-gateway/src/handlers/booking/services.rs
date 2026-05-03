//! Bookable service endpoints + service-resource M2M.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use booking::{
    AssignServiceResourcesCommand, AssignServiceResourcesUseCase, CreateServiceCommand,
    CreateServiceUseCase, DeactivateServiceUseCase, ListServicesUseCase, ServiceId,
    ServiceResponse, UpdateServiceCommand, UpdateServiceUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListServicesQuery {
    pub store_id: Uuid,
    pub include_inactive: Option<bool>,
}

pub async fn list_services_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListServicesQuery>,
) -> Result<Json<Vec<ServiceResponse>>, Response> {
    require_permission(&ctx, "booking:read_service")?;
    let only_active = !params.include_inactive.unwrap_or(false);
    let use_case = ListServicesUseCase::new(state.booking_service_repo());
    let services = use_case
        .execute(params.store_id, only_active)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(services.iter().map(ServiceResponse::from).collect()))
}

pub async fn create_service_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateServiceCommand>,
) -> Result<Json<ServiceResponse>, Response> {
    require_permission(&ctx, "booking:write_service")?;
    let use_case = CreateServiceUseCase::new(state.booking_service_repo());
    let service = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ServiceResponse::from(&service)))
}

pub async fn update_service_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateServiceCommand>,
) -> Result<Json<ServiceResponse>, Response> {
    require_permission(&ctx, "booking:write_service")?;
    let use_case = UpdateServiceUseCase::new(state.booking_service_repo());
    let service = use_case
        .execute(ServiceId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ServiceResponse::from(&service)))
}

pub async fn deactivate_service_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, Response> {
    require_permission(&ctx, "booking:write_service")?;
    let use_case = DeactivateServiceUseCase::new(state.booking_service_repo());
    use_case
        .execute(ServiceId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn assign_service_resources_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<AssignServiceResourcesCommand>,
) -> Result<axum::http::StatusCode, Response> {
    require_permission(&ctx, "booking:write_service")?;
    let use_case =
        AssignServiceResourcesUseCase::new(state.booking_service_repo(), state.resource_repo());
    use_case
        .execute(ServiceId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
