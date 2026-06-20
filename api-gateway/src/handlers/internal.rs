//! Internal service-to-service endpoints (not part of the public tenant API).
//!
//! `POST /internal/issue-impersonation-token` mints a tenant-audience
//! impersonation token (with an RFC 8693 `act` claim) signed with this
//! gateway's JWT_SECRET. The backoffice-api calls it instead of signing
//! locally, so the tenant signing key never leaves api-gateway (impersonation
//! v2 — see docs/backoffice-api.md).
//!
//! Auth: a shared `INTERNAL_SERVICE_SECRET` sent in the `X-Internal-Service-Token`
//! header. The endpoint is meant to be reachable only on the internal network
//! (compose/k8s); mTLS is the eventual transport-level hardening (infra TODO).

use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use identity::infrastructure::JwtTokenService;
use identity::{TokenService, UserId, UserRepository};

/// Header carrying the shared internal-service secret.
const INTERNAL_TOKEN_HEADER: &str = "X-Internal-Service-Token";

/// Impersonation token lifetime echoed back to the caller (seconds).
const IMPERSONATION_TOKEN_EXPIRES_IN: i64 = 900;

/// State for the internal router: the token minter, the tenant user repository
/// (to load the impersonated user's claims), and the shared secret used to
/// authenticate callers. Kept separate from `AppState` so these endpoints don't
/// ride the tenant auth middleware.
#[derive(Clone)]
pub struct InternalState {
    pub token_service: Arc<JwtTokenService>,
    pub user_repo: Arc<dyn UserRepository>,
    pub internal_secret: Arc<str>,
}

#[derive(Debug, Deserialize)]
pub struct IssueImpersonationRequest {
    /// The tenant user being impersonated (`sub`).
    pub tenant_user_id: Uuid,
    /// The backoffice operator (the `act` actor).
    pub operator_id: Uuid,
    pub operator_email: String,
}

#[derive(Debug, Serialize)]
pub struct IssueImpersonationResponse {
    pub access_token: String,
    pub expires_in: i64,
}

/// POST /internal/issue-impersonation-token
pub async fn issue_impersonation_token_handler(
    State(state): State<InternalState>,
    headers: HeaderMap,
    Json(body): Json<IssueImpersonationRequest>,
) -> Result<Json<IssueImpersonationResponse>, StatusCode> {
    authenticate(&headers, &state.internal_secret)?;

    // Load the impersonated tenant user so the minted token carries the same
    // claims (username/email/permissions/org) a normal login would — otherwise
    // the gateway's own auth middleware rejects it.
    let user = state
        .user_repo
        .find_by_id(UserId::from_uuid(body.tenant_user_id))
        .await
        .map_err(|e| {
            tracing::error!("tenant user lookup failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let store_permissions = state
        .user_repo
        .get_all_store_permissions(*user.id())
        .await
        .map_err(|e| {
            tracing::error!("tenant permission lookup failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let token = state
        .token_service
        .issue_impersonation_token(
            &user,
            &store_permissions,
            body.operator_id,
            &body.operator_email,
        )
        .map_err(|e| {
            tracing::error!("failed to mint impersonation token: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(IssueImpersonationResponse {
        access_token: token,
        expires_in: IMPERSONATION_TOKEN_EXPIRES_IN,
    }))
}

/// Rejects with 401 unless the `X-Internal-Service-Token` header matches the
/// configured secret. Constant-time compare to avoid leaking the secret via
/// response timing.
fn authenticate(headers: &HeaderMap, expected: &str) -> Result<(), StatusCode> {
    let provided = headers
        .get(INTERNAL_TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Length-aware constant-time byte comparison (no early return on mismatch).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
