//! Organization endpoints — list/register/get/update/suspend/activate plus
//! the bundled detail with plan + domains + branding.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use tenancy::{
    ActivateOrganizationUseCase, GetBrandingUseCase, GetOrganizationUseCase, GetPlanUseCase,
    ListDomainsUseCase, ListOrganizationsUseCase, OrganizationBrandingResponse,
    OrganizationDetailResponse, OrganizationDomainResponse, OrganizationId,
    OrganizationPlanResponse, OrganizationResponse, RegisterOrganizationCommand,
    RegisterOrganizationUseCase, SuspendOrganizationUseCase, UpdateOrganizationCommand,
    UpdateOrganizationUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::require_org_match;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListOrganizationsQuery {
    pub include_inactive: Option<bool>,
}

pub async fn list_organizations_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListOrganizationsQuery>,
) -> Result<Json<Vec<OrganizationResponse>>, Response> {
    require_permission(&ctx, "tenancy:read_org")?;
    let only_active = !params.include_inactive.unwrap_or(false);

    // super_admin sees the full directory; everyone else (org_admin, ...)
    // gets their own organization only — keep the cross-org leak shut at
    // this layer rather than relying on a downstream filter.
    if ctx.is_super_admin() {
        let use_case = ListOrganizationsUseCase::new(state.organization_repo());
        let orgs = use_case
            .execute(only_active)
            .await
            .map_err(|e| AppError::from(e).into_response())?;
        return Ok(Json(orgs.iter().map(OrganizationResponse::from).collect()));
    }
    let Some(my_org_uuid) = ctx.organization_id() else {
        return Ok(Json(Vec::new()));
    };
    let org = GetOrganizationUseCase::new(state.organization_repo())
        .execute(OrganizationId::from_uuid(my_org_uuid))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    if only_active && org.status() != tenancy::OrganizationStatus::Active {
        return Ok(Json(Vec::new()));
    }
    Ok(Json(vec![OrganizationResponse::from(&org)]))
}

pub async fn register_organization_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<RegisterOrganizationCommand>,
) -> Result<Json<OrganizationDetailResponse>, Response> {
    require_permission(&ctx, "tenancy:write_org")?;
    let use_case =
        RegisterOrganizationUseCase::new(state.organization_repo(), state.organization_plan_repo());
    let (org, plan) = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationDetailResponse {
        organization: OrganizationResponse::from(&org),
        plan: Some(OrganizationPlanResponse::from(&plan)),
        domains: Vec::new(),
        branding: None,
    }))
}

pub async fn get_organization_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<OrganizationDetailResponse>, Response> {
    require_permission(&ctx, "tenancy:read_org")?;
    require_org_match(&ctx, id)?;
    let org_id = OrganizationId::from_uuid(id);
    let org = GetOrganizationUseCase::new(state.organization_repo())
        .execute(org_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    let plan = GetPlanUseCase::new(state.organization_plan_repo())
        .execute(org_id)
        .await
        .ok();
    let domains = ListDomainsUseCase::new(state.organization_domain_repo())
        .execute(org_id)
        .await
        .unwrap_or_default();
    let branding = GetBrandingUseCase::new(state.organization_branding_repo())
        .execute(org_id)
        .await
        .ok();
    Ok(Json(OrganizationDetailResponse {
        organization: OrganizationResponse::from(&org),
        plan: plan.as_ref().map(OrganizationPlanResponse::from),
        domains: domains
            .iter()
            .map(OrganizationDomainResponse::from)
            .collect(),
        branding: branding.as_ref().map(OrganizationBrandingResponse::from),
    }))
}

pub async fn update_organization_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateOrganizationCommand>,
) -> Result<Json<OrganizationResponse>, Response> {
    require_permission(&ctx, "tenancy:write_org")?;
    require_org_match(&ctx, id)?;
    let use_case = UpdateOrganizationUseCase::new(state.organization_repo());
    let org = use_case
        .execute(OrganizationId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationResponse::from(&org)))
}

pub async fn suspend_organization_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<OrganizationResponse>, Response> {
    // suspend/activate stays super_admin only by perm assignment — no
    // org_admin in v1.0/v1.1 should be able to suspend their own tenant.
    require_permission(&ctx, "tenancy:suspend_org")?;
    let use_case = SuspendOrganizationUseCase::new(state.organization_repo());
    let org = use_case
        .execute(OrganizationId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationResponse::from(&org)))
}

pub async fn activate_organization_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<OrganizationResponse>, Response> {
    require_permission(&ctx, "tenancy:suspend_org")?;
    let use_case = ActivateOrganizationUseCase::new(state.organization_repo());
    let org = use_case
        .execute(OrganizationId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationResponse::from(&org)))
}
