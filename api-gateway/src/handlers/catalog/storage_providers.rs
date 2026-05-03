// Image storage provider handlers — CUD restricted to super_admin.

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
use crate::middleware::org_scope::verify_store_in_org;
use crate::middleware::permission::{require_permission, require_super_admin};
use crate::state::AppState;
use catalog::{
    ConfigureStorageProviderCommand, ConfigureStorageProviderUseCase, DeleteStorageProviderUseCase,
    ListStorageProvidersUseCase, StorageProviderResponse, UpdateStorageProviderCommand,
    UpdateStorageProviderUseCase,
};

#[derive(Debug, Deserialize)]
pub struct StoreScopedQuery {
    pub store_id: Uuid,
}

pub async fn configure_storage_provider_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<ConfigureStorageProviderCommand>,
) -> Result<(StatusCode, Json<StorageProviderResponse>), Response> {
    require_super_admin(&ctx)?;
    let uc = ConfigureStorageProviderUseCase::new(state.image_storage_provider_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn list_storage_providers_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<StoreScopedQuery>,
) -> Result<Json<Vec<StorageProviderResponse>>, Response> {
    require_permission(&ctx, "image_storage_providers:read")?;
    verify_store_in_org(state.pool(), &ctx, q.store_id).await?;
    let uc = ListStorageProvidersUseCase::new(state.image_storage_provider_repo());
    let resp = uc
        .execute(q.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn update_storage_provider_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<UpdateStorageProviderCommand>,
) -> Result<Json<StorageProviderResponse>, Response> {
    require_super_admin(&ctx)?;
    cmd.provider_id = id;
    let uc = UpdateStorageProviderUseCase::new(state.image_storage_provider_repo());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn delete_storage_provider_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_super_admin(&ctx)?;
    let uc = DeleteStorageProviderUseCase::new(state.image_storage_provider_repo());
    uc.execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(StatusCode::NO_CONTENT)
}
