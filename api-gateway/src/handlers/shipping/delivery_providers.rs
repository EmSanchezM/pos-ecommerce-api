// DeliveryProvider handlers — CUD restricted to super_admin.

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
use crate::middleware::org_scope::verify_store_in_org;
use crate::middleware::permission::{require_permission, require_super_admin};
use crate::state::AppState;
use shipping::{
    ConfigureDeliveryProviderCommand, ConfigureDeliveryProviderUseCase,
    DeleteDeliveryProviderUseCase, DeliveryProviderResponse, ListDeliveryProvidersUseCase,
    UpdateDeliveryProviderCommand, UpdateDeliveryProviderUseCase,
};

pub async fn configure_delivery_provider_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<ConfigureDeliveryProviderCommand>,
) -> Result<(StatusCode, Json<DeliveryProviderResponse>), Response> {
    require_super_admin(&ctx)?;
    let uc = ConfigureDeliveryProviderUseCase::new(state.delivery_provider_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn list_delivery_providers_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<StoreScopedQuery>,
) -> Result<Json<Vec<DeliveryProviderResponse>>, Response> {
    require_permission(&ctx, "delivery_providers:read")?;
    verify_store_in_org(state.pool(), &ctx, q.store_id).await?;
    let uc = ListDeliveryProvidersUseCase::new(state.delivery_provider_repo());
    let resp = uc
        .execute(q.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn update_delivery_provider_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<UpdateDeliveryProviderCommand>,
) -> Result<Json<DeliveryProviderResponse>, Response> {
    require_super_admin(&ctx)?;
    cmd.provider_id = id;
    let uc = UpdateDeliveryProviderUseCase::new(state.delivery_provider_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn delete_delivery_provider_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_super_admin(&ctx)?;
    let uc = DeleteDeliveryProviderUseCase::new(state.delivery_provider_repo());
    uc.execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(StatusCode::NO_CONTENT)
}
