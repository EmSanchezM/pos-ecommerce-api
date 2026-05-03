//! Asset endpoints (vehicles, equipment, appliances) + history.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use service_orders::{
    AssetId, AssetResponse, AssetType, DeactivateAssetUseCase, GetAssetUseCase,
    GetAssetWithHistoryUseCase, ListAssetsUseCase, RegisterAssetCommand, RegisterAssetUseCase,
    ServiceOrderResponse, UpdateAssetCommand, UpdateAssetUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::{require_feature, verify_store_in_org};
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListAssetsQuery {
    pub store_id: Uuid,
    pub include_inactive: Option<bool>,
    pub asset_type: Option<AssetType>,
}

pub async fn list_assets_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListAssetsQuery>,
) -> Result<Json<Vec<AssetResponse>>, Response> {
    require_permission(&ctx, "service_orders:read_asset")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    verify_store_in_org(state.pool(), &ctx, params.store_id).await?;
    let only_active = !params.include_inactive.unwrap_or(false);
    let use_case = ListAssetsUseCase::new(state.service_asset_repo());
    let assets = use_case
        .execute(params.store_id, only_active, params.asset_type)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(assets.iter().map(AssetResponse::from).collect()))
}

pub async fn get_asset_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetResponse>, Response> {
    require_permission(&ctx, "service_orders:read_asset")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    let use_case = GetAssetUseCase::new(state.service_asset_repo());
    let asset = use_case
        .execute(AssetId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AssetResponse::from(&asset)))
}

#[derive(Debug, serde::Serialize)]
pub struct AssetHistoryResponse {
    pub asset: AssetResponse,
    pub history: Vec<ServiceOrderResponse>,
}

pub async fn get_asset_history_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetHistoryResponse>, Response> {
    require_permission(&ctx, "service_orders:read_asset")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    let use_case =
        GetAssetWithHistoryUseCase::new(state.service_asset_repo(), state.service_order_repo());
    let (asset, history) = use_case
        .execute(AssetId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AssetHistoryResponse {
        asset: AssetResponse::from(&asset),
        history: history.iter().map(ServiceOrderResponse::from).collect(),
    }))
}

pub async fn register_asset_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<RegisterAssetCommand>,
) -> Result<Json<AssetResponse>, Response> {
    require_permission(&ctx, "service_orders:write_asset")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    verify_store_in_org(state.pool(), &ctx, cmd.store_id).await?;
    let use_case = RegisterAssetUseCase::new(state.service_asset_repo());
    let asset = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AssetResponse::from(&asset)))
}

pub async fn update_asset_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateAssetCommand>,
) -> Result<Json<AssetResponse>, Response> {
    require_permission(&ctx, "service_orders:write_asset")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    let use_case = UpdateAssetUseCase::new(state.service_asset_repo());
    let asset = use_case
        .execute(AssetId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AssetResponse::from(&asset)))
}

pub async fn deactivate_asset_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, Response> {
    require_permission(&ctx, "service_orders:write_asset")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    let use_case = DeactivateAssetUseCase::new(state.service_asset_repo());
    use_case
        .execute(AssetId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
