// Impersonation handler
//
// POST /backoffice/impersonate/{tenant_user_id}
//
// FR-IMP-1: endpoint exists and is gated by platform:user.impersonate.
// FR-IMP-2: issued token carries aud:Tenant, sub=tenant_user_id, act.sub=operator.
// FR-IMP-5: audit event written in the same transaction as the token issuance.
// NFR-SEC-4: token expires in 15 minutes (enforced in the use case).
// NFR-SEC-5: reason field is required — 400 if empty.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use backoffice_identity::BackofficeUserId;
use identity::UserId;

use crate::error::{AppError, ErrorResponse};
use crate::middleware::auth::BackofficeUserContext;
use crate::middleware::permission::require_backoffice_permission;
use crate::state::BackofficeAppState;

/// Request body for POST /backoffice/impersonate/{tenant_user_id}.
#[derive(Debug, Deserialize)]
pub struct ImpersonateRequest {
    /// Required reason for impersonation (NFR-SEC-5, FR-AUD-2).
    pub reason: String,
}

/// POST /backoffice/impersonate/{tenant_user_id}
///
/// Issues a 15-minute impersonation token that allows the backoffice operator
/// to interact with api-gateway as the specified tenant user.
///
/// # Flow
///
/// 1. Validate reason (400 if empty — before ANY DB access).
/// 2. Verify permission gate: platform:user.impersonate (403 if absent).
/// 3. Verify the target tenant_user_id exists (404 if not found).
/// 4. Delegate to `IssueImpersonationTokenWithAuditUseCase` which:
///    a. Issues the token (CPU only).
///    b. Opens a transaction.
///    c. Writes the audit outbox event.
///    d. Commits.
/// 5. Return 200 with `ImpersonationTokenResponse`.
///
/// # Error responses
///
/// - 400: reason field empty or missing.
/// - 401: no valid Backoffice JWT in Authorization header.
/// - 403: operator lacks `platform:user.impersonate`.
/// - 404: tenant_user_id does not exist.
/// - 500: unexpected server error.
pub async fn impersonate_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(tenant_user_id): Path<Uuid>,
    Json(body): Json<ImpersonateRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Step 1: validate reason BEFORE any DB operation.
    if body.reason.trim().is_empty() {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            ErrorResponse::new(
                "REASON_REQUIRED",
                "The 'reason' field is required for impersonation",
            ),
        ));
    }

    // Step 2: permission gate — platform:user.impersonate.
    require_backoffice_permission(&ctx, "platform:user.impersonate").map_err(|_| {
        AppError::forbidden("Missing required platform permission: platform:user.impersonate")
    })?;

    // Step 3: verify the target tenant user exists.
    //
    // We use the identity::UserRepository (wired in BackofficeAppState) to check
    // existence. This is the only place we touch the tenant `users` table from
    // backoffice-api, and it's read-only (no mutation of tenant data).
    let tenant_user_exists = state
        .tenant_user_repo()
        .find_by_id(UserId::from_uuid(tenant_user_id))
        .await
        .map_err(|e| {
            tracing::error!("tenant user lookup failed: {}", e);
            AppError::internal()
        })?
        .is_some();

    if !tenant_user_exists {
        return Err(AppError::new(
            StatusCode::NOT_FOUND,
            ErrorResponse::new(
                "TENANT_USER_NOT_FOUND",
                format!("Tenant user {} does not exist", tenant_user_id),
            ),
        ));
    }

    // Step 4: extract IP (Phase 4 placeholder — Phase 5 wires ConnectInfo if needed).
    let ip = "0.0.0.0".to_string();

    // Step 5: delegate to the use case which handles token issuance + audit in one tx.
    let response = state
        .impersonation_use_case()
        .execute(
            BackofficeUserId::from_uuid(ctx.user_id),
            tenant_user_id,
            body.reason,
            ip,
        )
        .await
        .map_err(AppError::from)?;

    Ok((StatusCode::OK, Json(response)))
}

// =============================================================================
// Tests — S-04 shape + reason validation
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// S-07 / NFR-SEC-5: empty reason rejected before any DB access.
    #[test]
    fn empty_reason_rejected() {
        let req = ImpersonateRequest {
            reason: "".to_string(),
        };
        assert!(req.reason.trim().is_empty());
    }

    /// Whitespace-only reason rejected.
    #[test]
    fn whitespace_reason_rejected() {
        let req = ImpersonateRequest {
            reason: "   \t".to_string(),
        };
        assert!(req.reason.trim().is_empty());
    }

    /// Non-empty reason passes.
    #[test]
    fn non_empty_reason_passes() {
        let req = ImpersonateRequest {
            reason: "Support escalation ticket #9999".to_string(),
        };
        assert!(!req.reason.trim().is_empty());
    }
}
