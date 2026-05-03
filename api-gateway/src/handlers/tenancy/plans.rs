use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use tenancy::{
    GetPlanUseCase, OrganizationId, OrganizationPlanResponse, SetFeatureFlagCommand,
    SetFeatureFlagUseCase, SetPlanCommand, SetPlanUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::require_org_match;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

pub async fn get_plan_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
) -> Result<Json<OrganizationPlanResponse>, Response> {
    require_permission(&ctx, "tenancy:read_plan")?;
    require_org_match(&ctx, org_id)?;
    let use_case = GetPlanUseCase::new(state.organization_plan_repo());
    let plan = use_case
        .execute(OrganizationId::from_uuid(org_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationPlanResponse::from(&plan)))
}

pub async fn set_plan_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
    Json(cmd): Json<SetPlanCommand>,
) -> Result<Json<OrganizationPlanResponse>, Response> {
    require_permission(&ctx, "tenancy:write_plan")?;
    let use_case = SetPlanUseCase::new(state.organization_repo(), state.organization_plan_repo());
    let plan = use_case
        .execute(OrganizationId::from_uuid(org_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationPlanResponse::from(&plan)))
}

pub async fn set_feature_flag_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
    Json(cmd): Json<SetFeatureFlagCommand>,
) -> Result<Json<OrganizationPlanResponse>, Response> {
    require_permission(&ctx, "tenancy:write_plan")?;
    let use_case = SetFeatureFlagUseCase::new(state.organization_plan_repo());
    let plan = use_case
        .execute(OrganizationId::from_uuid(org_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationPlanResponse::from(&plan)))
}
