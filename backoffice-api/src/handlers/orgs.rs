// Organization admin handlers
//
// GET /backoffice/orgs — list every organization on the platform, gated by
// the `platform:org.list` permission. Unlike api-gateway's tenancy handler,
// the backoffice operator always sees the full cross-org directory: this is
// the platform-owner view, not a tenant-scoped one.

use axum::{
    Extension, Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tenancy::{ListOrganizationsUseCase, OrganizationResponse};

use crate::error::AppError;
use crate::middleware::auth::BackofficeUserContext;
use crate::middleware::permission::require_backoffice_permission;
use crate::state::BackofficeAppState;

/// Query parameters for `GET /backoffice/orgs`.
#[derive(Debug, Deserialize)]
pub struct ListOrgsQuery {
    /// When true, suspended/inactive organizations are included in the result.
    /// Defaults to false — the listing shows only active orgs.
    pub include_inactive: Option<bool>,
}

/// GET /backoffice/orgs
///
/// Lists organizations across every tenant. Requires `platform:org.list`.
/// Returns 403 if the operator lacks the permission.
pub async fn list_orgs_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Query(params): Query<ListOrgsQuery>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:org.list")?;

    let only_active = !params.include_inactive.unwrap_or(false);

    let use_case = ListOrganizationsUseCase::new(state.organization_repo());
    let orgs = use_case
        .execute(only_active)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    let body: Vec<OrganizationResponse> = orgs.iter().map(OrganizationResponse::from).collect();
    Ok(Json(body))
}
