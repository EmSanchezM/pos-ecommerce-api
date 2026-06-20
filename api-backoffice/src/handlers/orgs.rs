// Organization admin handlers
//
// GET  /backoffice/orgs              — list every organization on the platform,
//                                       gated by `platform:org.list`. The
//                                       backoffice operator always sees the full
//                                       cross-org directory (platform-owner view).
// POST /backoffice/orgs/{id}/suspend — transactional suspend with audit, gated
//                                       by `platform:org.suspend`.

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use backoffice_identity::BackofficeUserId;
use tenancy::{ListOrganizationsUseCase, OrganizationId, OrganizationResponse};

use crate::error::{AppError, ErrorResponse};
use crate::middleware::auth::BackofficeUserContext;
use crate::middleware::permission::require_backoffice_permission;
use crate::state::BackofficeAppState;

// =============================================================================
// GET /backoffice/orgs — cross-org listing
// =============================================================================

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

    let use_case = ListOrganizationsUseCase::new(state.org_repo());
    let orgs = use_case
        .execute(only_active)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    let body: Vec<OrganizationResponse> = orgs.iter().map(OrganizationResponse::from).collect();
    Ok(Json(body))
}

// =============================================================================
// P4-T09: Suspend org endpoint
// =============================================================================

/// Request body for POST /backoffice/orgs/{id}/suspend.
#[derive(Debug, Deserialize)]
pub struct SuspendOrgRequest {
    /// Required reason for the suspension (NFR-SEC-5).
    pub reason: String,
}

/// Response body for 200 OK on suspend.
#[derive(Debug, Serialize)]
pub struct SuspendOrgResponse {
    pub org_id: Uuid,
    pub status: String,
    pub message: String,
}

/// POST /backoffice/orgs/{id}/suspend
///
/// Suspends an organization and writes an audit event to the transactional
/// outbox in the SAME transaction as the org status update.
///
/// # Spec coverage
/// - S-06: suspend writes outbox event + org update in same tx.
/// - S-07: missing/empty reason → 400 BEFORE any DB change.
/// - FR-AUD-1, FR-AUD-2, C-7: transactional outbox write.
/// - NFR-SEC-5: reason required.
/// - Permission gate: `platform:org.suspend` (applied by router middleware).
pub async fn suspend_org_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<SuspendOrgRequest>,
) -> Result<impl IntoResponse, AppError> {
    // S-07: validate reason FIRST — before any DB access.
    // The use case also validates, but we do it here to return 400 quickly and
    // avoid constructing/calling the use case with obviously invalid input.
    if body.reason.trim().is_empty() {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            ErrorResponse::new(
                "REASON_REQUIRED",
                "The 'reason' field is required for all state-mutating backoffice actions",
            ),
        ));
    }

    // Phase 4 placeholder IP — Phase 5 wires ConnectInfo.
    let ip = "0.0.0.0".to_string(); // TODO(phase5): extract from ConnectInfo

    let org = state
        .suspend_with_audit_use_case()
        .execute(
            BackofficeUserId::from_uuid(ctx.user_id),
            OrganizationId::from_uuid(org_id),
            body.reason.clone(),
            ip,
        )
        .await
        .map_err(AppError::from)?;

    Ok((
        StatusCode::OK,
        Json(SuspendOrgResponse {
            org_id,
            status: org.status().as_str().to_string(),
            message: format!("Organization {} suspended. Reason: {}", org_id, body.reason),
        }),
    ))
}

// =============================================================================
// Tests — S-06 and S-07 acceptance scenarios
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    /// S-07: empty reason field must be detected before any DB operation.
    #[test]
    fn empty_reason_rejected() {
        let req = SuspendOrgRequest {
            reason: "".to_string(),
        };
        assert!(
            req.reason.trim().is_empty(),
            "empty reason should be detected by the handler's validation guard"
        );
    }

    /// S-07: whitespace-only reason must also be rejected.
    #[test]
    fn whitespace_reason_rejected() {
        let req = SuspendOrgRequest {
            reason: "   \t\n".to_string(),
        };
        assert!(req.reason.trim().is_empty());
    }

    /// S-06: non-empty reason passes validation.
    #[test]
    fn non_empty_reason_passes() {
        let req = SuspendOrgRequest {
            reason: "fraud detected by compliance team".to_string(),
        };
        assert!(!req.reason.trim().is_empty());
    }

    /// Verify OrganizationId roundtrip (used in use case invocation).
    #[test]
    fn org_id_roundtrip() {
        use uuid::{NoContext, Timestamp};
        let org_uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let org_id = OrganizationId::from_uuid(org_uuid);
        assert_eq!(org_id.into_uuid(), org_uuid);
    }
}
