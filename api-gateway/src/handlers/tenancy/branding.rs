use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use tenancy::{
    GetBrandingUseCase, OrganizationBrandingResponse, OrganizationId, UpsertBrandingCommand,
    UpsertBrandingUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::require_org_match;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

pub async fn get_branding_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
) -> Result<Json<OrganizationBrandingResponse>, Response> {
    require_permission(&ctx, "tenancy:read_branding")?;
    require_org_match(&ctx, org_id)?;
    let use_case = GetBrandingUseCase::new(state.organization_branding_repo());
    let branding = use_case
        .execute(OrganizationId::from_uuid(org_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationBrandingResponse::from(&branding)))
}

pub async fn upsert_branding_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
    Json(cmd): Json<UpsertBrandingCommand>,
) -> Result<Json<OrganizationBrandingResponse>, Response> {
    require_permission(&ctx, "tenancy:write_branding")?;
    require_org_match(&ctx, org_id)?;
    let use_case = UpsertBrandingUseCase::new(
        state.organization_repo(),
        state.organization_branding_repo(),
    );
    let branding = use_case
        .execute(OrganizationId::from_uuid(org_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationBrandingResponse::from(&branding)))
}
