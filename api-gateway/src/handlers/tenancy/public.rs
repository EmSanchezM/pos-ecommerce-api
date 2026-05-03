//! Public organization lookup — no auth. Used by the storefront to resolve
//! `tienda.acme.com` (or `?org=acme`) into an org id + branding so the page
//! can theme itself before login.

use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use tenancy::{
    FindOrganizationByDomainUseCase, GetBrandingUseCase, GetOrganizationBySlugUseCase,
    OrganizationBrandingResponse, PublicOrganizationResponse,
};

use crate::error::AppError;
use crate::state::AppState;

pub async fn public_org_by_slug_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<PublicOrganizationResponse>, Response> {
    let org = GetOrganizationBySlugUseCase::new(state.organization_repo())
        .execute(&slug)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    let branding = GetBrandingUseCase::new(state.organization_branding_repo())
        .execute(org.id())
        .await
        .ok();
    Ok(Json(PublicOrganizationResponse {
        id: org.id().into_uuid(),
        name: org.name().to_string(),
        slug: org.slug().to_string(),
        branding: branding.as_ref().map(OrganizationBrandingResponse::from),
    }))
}

pub async fn public_org_by_domain_handler(
    State(state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<Json<PublicOrganizationResponse>, Response> {
    let org = FindOrganizationByDomainUseCase::new(
        state.organization_domain_repo(),
        state.organization_repo(),
    )
    .execute(&domain)
    .await
    .map_err(|e| AppError::from(e).into_response())?;
    let branding = GetBrandingUseCase::new(state.organization_branding_repo())
        .execute(org.id())
        .await
        .ok();
    Ok(Json(PublicOrganizationResponse {
        id: org.id().into_uuid(),
        name: org.name().to_string(),
        slug: org.slug().to_string(),
        branding: branding.as_ref().map(OrganizationBrandingResponse::from),
    }))
}
