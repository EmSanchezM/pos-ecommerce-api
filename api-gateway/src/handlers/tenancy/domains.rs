use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use tenancy::{
    DeleteDomainUseCase, ListDomainsUseCase, OrganizationDomainId, OrganizationDomainResponse,
    OrganizationId, RegisterDomainCommand, RegisterDomainUseCase, SetPrimaryDomainUseCase,
    VerifyDomainUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::require_org_match;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

pub async fn list_domains_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<OrganizationDomainResponse>>, Response> {
    require_permission(&ctx, "tenancy:read_domain")?;
    require_org_match(&ctx, org_id)?;
    let use_case = ListDomainsUseCase::new(state.organization_domain_repo());
    let domains = use_case
        .execute(OrganizationId::from_uuid(org_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        domains
            .iter()
            .map(OrganizationDomainResponse::from)
            .collect(),
    ))
}

pub async fn register_domain_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
    Json(cmd): Json<RegisterDomainCommand>,
) -> Result<Json<OrganizationDomainResponse>, Response> {
    require_permission(&ctx, "tenancy:write_domain")?;
    require_org_match(&ctx, org_id)?;
    let use_case =
        RegisterDomainUseCase::new(state.organization_repo(), state.organization_domain_repo());
    let domain = use_case
        .execute(OrganizationId::from_uuid(org_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationDomainResponse::from(&domain)))
}

pub async fn verify_domain_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((org_id, domain_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<OrganizationDomainResponse>, Response> {
    require_permission(&ctx, "tenancy:verify_domain")?;
    require_org_match(&ctx, org_id)?;
    let use_case = VerifyDomainUseCase::new(state.organization_domain_repo());
    let domain = use_case
        .execute(OrganizationDomainId::from_uuid(domain_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationDomainResponse::from(&domain)))
}

pub async fn set_primary_domain_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((org_id, domain_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<OrganizationDomainResponse>, Response> {
    require_permission(&ctx, "tenancy:write_domain")?;
    require_org_match(&ctx, org_id)?;
    let use_case = SetPrimaryDomainUseCase::new(state.organization_domain_repo());
    let domain = use_case
        .execute(
            OrganizationId::from_uuid(org_id),
            OrganizationDomainId::from_uuid(domain_id),
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(OrganizationDomainResponse::from(&domain)))
}

pub async fn delete_domain_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((org_id, domain_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, Response> {
    require_permission(&ctx, "tenancy:write_domain")?;
    require_org_match(&ctx, org_id)?;
    let use_case = DeleteDomainUseCase::new(state.organization_domain_repo());
    use_case
        .execute(OrganizationDomainId::from_uuid(domain_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
