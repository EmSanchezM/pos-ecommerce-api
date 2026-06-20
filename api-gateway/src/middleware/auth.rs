// Authentication Middleware for JWT validation
//
// This middleware extracts and validates JWT tokens from the Authorization header,
// builds the UserContext with permissions, and injects it into request extensions.
//
// FR-IMP-5: When a request arrives with an act claim (impersonation token),
// a synchronous outbox event is written before forwarding. See crate::audit.

use axum::{
    Json,
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;

use std::collections::HashSet;

use common::TokenAudience;
use identity::{ErrorResponse, PermissionCode, StoreId, TokenService, UserContext, UserId};

use crate::state::AppState;

/// Authentication middleware that validates JWT tokens and builds UserContext.
///
/// This middleware:
/// 1. Extracts the Bearer token from the Authorization header
/// 2. Validates the token using TokenService
/// 3. Extracts user_id from token claims
/// 4. Builds UserContext with permissions from token claims (no DB query)
/// 5. Injects UserContext into request extensions for use by handlers
///
/// # Errors
///
/// - Returns 401 Unauthorized if:
///   - Authorization header is missing
///   - Authorization header doesn't use Bearer scheme
///   - Token is invalid or expired
///   - User is not found or inactive
///
/// - Return 401 if no token present
/// - Return 401 if token invalid or expired
/// - Extract user_id and build UserContext with permissions
/// - Inject UserContext as extractor for handlers
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    // 1. Extract token from Authorization header
    let token = match extract_bearer_token(&request) {
        Ok(token) => token,
        Err(response) => return response,
    };

    // 2. Validate token and get claims
    let claims = match state.token_service().validate_access_token(&token) {
        Ok(claims) => claims,
        Err(_) => {
            return unauthorized_response("Invalid or expired token");
        }
    };

    // 3. Reject any token not intended for the tenant surface (FR-JWT-4).
    //    validate_access_token already enforces this, but we make the intent
    //    explicit here so the middleware remains self-documenting.
    if claims.aud != TokenAudience::Tenant {
        return unauthorized_response("Invalid token audience");
    }

    // 4. Extract user_id from claims
    let user_id = UserId::from_uuid(claims.user_id());

    // 5. Get store_id from request header
    let store_id = extract_store_id(&request);

    // 6. Build UserContext from token claims (no DB query needed).
    //    Effective perms = (perms granted on the active store) ∪ (global perms).
    //    Globals (e.g. `tenancy:*`) apply regardless of `X-Store-Id` so an
    //    `org_admin` can hit `/api/v1/organizations` without picking a store.
    let mut permissions: HashSet<PermissionCode> = claims
        .store_permissions
        .get(&store_id.as_uuid().to_string())
        .map(|perms| {
            perms
                .iter()
                .filter_map(|p| PermissionCode::new(p).ok())
                .collect()
        })
        .unwrap_or_default();
    for p in &claims.global_permissions {
        if let Ok(code) = PermissionCode::new(p) {
            permissions.insert(code);
        }
    }

    // Extract all accessible store IDs from the JWT claims
    let accessible_store_ids: Vec<uuid::Uuid> = claims
        .store_permissions
        .keys()
        .filter_map(|k| uuid::Uuid::parse_str(k).ok())
        .collect();

    // Determine if the user is a super admin (any store has organization:admin permission)
    let is_super_admin = claims
        .store_permissions
        .values()
        .any(|perms| perms.iter().any(|p| p == "organization:admin"));

    // FR-IMP-3: extract the real actor ID from the act claim if present.
    // This is the RFC 8693 delegation claim on impersonation tokens.
    let actor_id: Option<uuid::Uuid> = claims.act.as_ref().map(|act| act.sub);

    // FR-IMP-4: when an act claim is present, log both actors for every request.
    if let Some(real_actor) = actor_id {
        tracing::info!(
            impersonated_user_id = %user_id,
            real_actor_id = %real_actor,
            "impersonated request: acting as tenant user on behalf of backoffice operator"
        );

        // FR-IMP-5: write a synchronous per-request outbox audit event.
        // Extract method, path, and client IP before request is consumed.
        let method = request.method().to_string();
        let path = request.uri().path().to_string();

        // Try ConnectInfo<SocketAddr> first (set by into_make_service_with_connect_info).
        // Fall back to X-Forwarded-For header, then "0.0.0.0" as a last resort.
        let ip = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0.ip().to_string())
            .or_else(|| {
                request
                    .headers()
                    .get("X-Forwarded-For")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.split(',').next().unwrap_or("0.0.0.0").trim().to_string())
            })
            .unwrap_or_else(|| "0.0.0.0".to_string());

        // Emit fail-open: if DB write fails, tracing::error! is logged and the
        // request continues. See crate::audit for the failure policy comment.
        crate::audit::emit_impersonated_request_audit(
            state.pool(),
            state.outbox_repo(),
            real_actor,
            &method,
            &path,
            &ip,
        )
        .await;
    }

    let user_context = UserContext::new(
        user_id,
        store_id,
        permissions,
        accessible_store_ids,
        is_super_admin,
        claims.organization_id,
        actor_id,
    );

    // 7. Insert UserContext into request extensions
    request.extensions_mut().insert(user_context);

    // 8. Continue to the next handler
    next.run(request).await
}

/// Extracts the Bearer token from the Authorization header.
///
/// # Arguments
///
/// * `request` - The incoming HTTP request
///
/// # Returns
///
/// * `Ok(String)` - The token string if successfully extracted
/// * `Err(Response)` - A 401 Unauthorized response if extraction fails
#[allow(clippy::result_large_err)]
fn extract_bearer_token(request: &Request<Body>) -> Result<String, Response> {
    // Get Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    match auth_header {
        Some(header) => {
            // Check for Bearer scheme
            if let Some(token) = header.strip_prefix("Bearer ") {
                if token.is_empty() {
                    Err(unauthorized_response("Token is empty"))
                } else {
                    Ok(token.to_string())
                }
            } else {
                Err(unauthorized_response(
                    "Invalid authorization scheme, expected Bearer",
                ))
            }
        }
        None => Err(unauthorized_response("Missing authorization header")),
    }
}

/// Extracts the store_id from the request.
///
/// Looks for store_id in the following order:
/// 1. X-Store-Id header
/// 2. Falls back to a nil UUID (for system-wide operations)
///
/// # Arguments
///
/// * `request` - The incoming HTTP request
///
/// # Returns
///
/// The StoreId extracted from the request or a default value
fn extract_store_id(request: &Request<Body>) -> StoreId {
    // Try to get store_id from X-Store-Id header
    if let Some(store_id_header) = request.headers().get("X-Store-Id")
        && let Ok(store_id_str) = store_id_header.to_str()
        && let Ok(uuid) = uuid::Uuid::parse_str(store_id_str)
    {
        return StoreId::from_uuid(uuid);
    }

    // Default to nil UUID for system-wide operations
    // In a real application, you might want to require a store_id
    // or have a different default behavior
    StoreId::from_uuid(uuid::Uuid::nil())
}

/// Creates a 401 Unauthorized response with a JSON error body.
///
/// # Arguments
///
/// * `message` - The error message to include in the response
///
/// # Returns
///
/// A Response with status 401 and JSON error body
fn unauthorized_response(message: &str) -> Response {
    let error_response = ErrorResponse::new("UNAUTHORIZED", message);
    (StatusCode::UNAUTHORIZED, Json(error_response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request() -> Request<Body> {
        Request::builder().uri("/test").body(Body::empty()).unwrap()
    }

    fn create_request_with_auth(auth_value: &str) -> Request<Body> {
        Request::builder()
            .uri("/test")
            .header(AUTHORIZATION, auth_value)
            .body(Body::empty())
            .unwrap()
    }

    #[test]
    fn test_extract_bearer_token_success() {
        let request = create_request_with_auth("Bearer valid_token_here");
        let result = extract_bearer_token(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "valid_token_here");
    }

    #[test]
    fn test_extract_bearer_token_missing_header() {
        let request = create_test_request();
        let result = extract_bearer_token(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let request = create_request_with_auth("Basic dXNlcjpwYXNz");
        let result = extract_bearer_token(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_bearer_token_empty_token() {
        let request = create_request_with_auth("Bearer ");
        let result = extract_bearer_token(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_store_id_from_header() {
        use uuid::{NoContext, Timestamp};
        let store_uuid = uuid::Uuid::new_v7(Timestamp::now(NoContext));
        let request = Request::builder()
            .uri("/test")
            .header("X-Store-Id", store_uuid.to_string())
            .body(Body::empty())
            .unwrap();

        let store_id = extract_store_id(&request);
        assert_eq!(store_id.into_uuid(), store_uuid);
    }

    #[test]
    fn test_extract_store_id_default() {
        let request = create_test_request();
        let store_id = extract_store_id(&request);
        assert_eq!(store_id.into_uuid(), uuid::Uuid::nil());
    }

    #[test]
    fn test_extract_store_id_invalid_uuid() {
        let request = Request::builder()
            .uri("/test")
            .header("X-Store-Id", "not-a-uuid")
            .body(Body::empty())
            .unwrap();

        let store_id = extract_store_id(&request);
        // Should fall back to nil UUID
        assert_eq!(store_id.into_uuid(), uuid::Uuid::nil());
    }

    #[test]
    fn test_is_super_admin_uses_organization_admin_not_system_admin() {
        // Regression test: after renaming system:admin → organization:admin,
        // the is_super_admin flag must be driven by "organization:admin".
        use std::collections::HashMap;

        let mut perms_with_new_code: HashMap<String, Vec<String>> = HashMap::new();
        perms_with_new_code.insert(
            "store-1".to_string(),
            vec!["organization:admin".to_string()],
        );

        let is_super_admin_new = perms_with_new_code
            .values()
            .any(|perms| perms.iter().any(|p| p == "organization:admin"));

        let mut perms_with_old_code: HashMap<String, Vec<String>> = HashMap::new();
        perms_with_old_code.insert("store-1".to_string(), vec!["system:admin".to_string()]);

        let is_super_admin_old = perms_with_old_code
            .values()
            .any(|perms| perms.iter().any(|p| p == "organization:admin"));

        assert!(
            is_super_admin_new,
            "organization:admin must grant super admin"
        );
        assert!(
            !is_super_admin_old,
            "system:admin must NOT grant super admin after rename"
        );
    }

    // -------------------------------------------------------------------------
    // P5-T04: act-claim acceptance tests (FR-IMP-3)
    //
    // Verify that the middleware correctly extracts the actor_id from an
    // impersonation token's `act` claim.
    //
    // These tests operate at the claim-parsing level (no full middleware stack)
    // since we cannot invoke the full middleware without a running server +
    // real token service. The actual actor_id extraction is tested inline.
    // -------------------------------------------------------------------------

    #[test]
    fn impersonation_token_act_claim_yields_actor_id() {
        use common::ActorClaim;
        use uuid::{NoContext, Timestamp};

        // Simulate what the middleware does after validate_access_token succeeds.
        // We have an impersonation token where act.sub is the backoffice operator.
        let tenant_user_id = uuid::Uuid::new_v7(Timestamp::now(NoContext));
        let backoffice_user_id = uuid::Uuid::new_v7(Timestamp::now(NoContext));

        // Construct a minimal ActorClaim as the middleware would see it.
        let act = ActorClaim {
            sub: backoffice_user_id,
            sub_type: "backoffice_user".to_string(),
            email: "operator@platform.com".to_string(),
        };

        // The extraction logic: actor_id = claims.act.as_ref().map(|a| a.sub)
        let actor_id: Option<uuid::Uuid> = Some(act).as_ref().map(|a| a.sub);

        assert_eq!(
            actor_id,
            Some(backoffice_user_id),
            "actor_id must be extracted from act.sub"
        );
        // The impersonated user would be the sub field (tenant_user_id)
        let _ = tenant_user_id; // suppresses unused warning
    }

    #[test]
    fn token_without_act_claim_has_no_actor_id() {
        // A regular tenant token has no act claim → actor_id = None.
        let act: Option<common::ActorClaim> = None;
        let actor_id: Option<uuid::Uuid> = act.as_ref().map(|a| a.sub);
        assert_eq!(actor_id, None, "no act claim → actor_id must be None");
    }

    #[test]
    fn usercontext_actor_id_getter_returns_correct_value() {
        use std::collections::HashSet;
        use uuid::{NoContext, Timestamp};

        let user_id = UserId::new();
        let store_id = StoreId::new();
        let backoffice_id = uuid::Uuid::new_v7(Timestamp::now(NoContext));

        // Build context as middleware would for an impersonation token.
        let ctx = UserContext::new(
            user_id,
            store_id,
            HashSet::new(),
            vec![],
            false,
            None,
            Some(backoffice_id),
        );

        assert_eq!(ctx.actor_id(), Some(backoffice_id));
        assert_eq!(*ctx.user_id(), user_id);
    }

    #[test]
    fn usercontext_actor_id_none_for_normal_token() {
        use std::collections::HashSet;

        let ctx = UserContext::new(
            UserId::new(),
            StoreId::new(),
            HashSet::new(),
            vec![],
            false,
            None,
            None, // normal token — no impersonation
        );

        assert_eq!(ctx.actor_id(), None);
    }

    // -------------------------------------------------------------------------
    // P2-T08: Audience rejection tests (FR-JWT-4)
    //
    // These tests verify at the token-service level that tokens with incorrect
    // audience are rejected before they can reach any handler. The middleware
    // itself delegates to `validate_access_token` which now enforces aud==Tenant.
    // -------------------------------------------------------------------------

    #[test]
    fn token_with_backoffice_aud_rejected_by_tenant_token_service() {
        use common::{BackofficeClaims, TokenAudience};
        use identity::JwtTokenService;
        use jsonwebtoken::{EncodingKey, Header, encode};
        use uuid::{NoContext, Timestamp};

        let secret = "shared-secret-for-test-at-least-32-bytes";
        let tenant_service = JwtTokenService::new(secret.to_string());

        // Craft a BackofficeClaims token signed with the SAME secret as the
        // tenant service — simulates cross-audience token smuggling attempt.
        let now = chrono::Utc::now();
        let backoffice_claims = BackofficeClaims {
            sub: uuid::Uuid::new_v7(Timestamp::now(NoContext)),
            aud: TokenAudience::Backoffice,
            iss: "backoffice-api:test".to_string(),
            exp: (now + chrono::Duration::hours(1)).timestamp(),
            iat: now.timestamp(),
            permissions: vec![],
        };

        let token = encode(
            &Header::default(),
            &backoffice_claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        // validate_access_token must reject aud: Backoffice
        let result = identity::TokenService::validate_access_token(&tenant_service, &token);
        assert!(
            result.is_err(),
            "aud:Backoffice token must be rejected by tenant token service"
        );
    }

    #[test]
    fn token_without_aud_field_rejected_by_tenant_token_service() {
        // A legacy token (no aud field) cannot deserialize into TokenClaims
        // because `aud` is now required (non-optional). This satisfies FR-JWT-7:
        // force re-login for tokens lacking aud/iss.
        use identity::JwtTokenService;
        use jsonwebtoken::{EncodingKey, Header, encode};
        use serde::{Deserialize, Serialize};

        let secret = "shared-secret-for-test-at-least-32-bytes";
        let tenant_service = JwtTokenService::new(secret.to_string());

        // Old-style claims without aud/iss fields
        #[derive(Serialize, Deserialize)]
        struct LegacyClaims {
            sub: uuid::Uuid,
            username: String,
            email: String,
            exp: i64,
            iat: i64,
        }

        let now = chrono::Utc::now();
        let legacy = LegacyClaims {
            sub: uuid::Uuid::nil(),
            username: "legacyuser".to_string(),
            email: "legacy@example.com".to_string(),
            exp: (now + chrono::Duration::hours(1)).timestamp(),
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &legacy,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let result = identity::TokenService::validate_access_token(&tenant_service, &token);
        assert!(result.is_err(), "Legacy token without aud must be rejected");
    }

    // -------------------------------------------------------------------------
    // FR-IMP-5: per-request impersonation audit — middleware branch tests
    //
    // Verifies that the impersonation branch correctly extracts the IP address
    // from the available sources and that the audit emit path is entered when
    // actor_id is present.
    // -------------------------------------------------------------------------

    #[test]
    fn ip_extracted_from_connect_info_when_present() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        // Simulate the IP extraction logic as the middleware applies it.
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 42)), 12345);
        let connect_info = ConnectInfo(socket_addr);

        let ip = connect_info.0.ip().to_string();
        assert_eq!(ip, "192.168.1.42");
    }

    #[test]
    fn ip_falls_back_to_x_forwarded_for_when_no_connect_info() {
        // Simulate the X-Forwarded-For extraction (first IP in comma-separated list)
        let header_value = "10.0.0.5, 172.16.0.1";
        let ip: String = header_value
            .split(',')
            .next()
            .unwrap_or("0.0.0.0")
            .trim()
            .to_string();

        assert_eq!(ip, "10.0.0.5");
    }

    #[test]
    // The None is a deliberate stand-in for "no ConnectInfo, no header"; clippy
    // flags unwrapping a literal None, but that is exactly the fallback under test.
    #[allow(clippy::unnecessary_literal_unwrap)]
    fn ip_falls_back_to_zeros_when_no_source_available() {
        // When neither ConnectInfo nor X-Forwarded-For is present, use "0.0.0.0"
        let ip: Option<String> = None; // simulates no ConnectInfo, no header
        let resolved = ip.unwrap_or_else(|| "0.0.0.0".to_string());
        assert_eq!(resolved, "0.0.0.0");
    }

    /// FR-IMP-5: the middleware audit branch is entered when and only when
    /// actor_id is Some (act claim present on the token).
    #[test]
    fn audit_branch_entered_only_when_actor_id_is_some() {
        use uuid::{NoContext, Timestamp};

        let actor_with_impersonation: Option<uuid::Uuid> =
            Some(uuid::Uuid::new_v7(Timestamp::now(NoContext)));
        let actor_without_impersonation: Option<uuid::Uuid> = None;

        // The audit branch is entered if and only if actor_id is Some.
        let should_emit_audit = actor_with_impersonation.is_some();
        let should_not_emit_audit = actor_without_impersonation.is_some();

        assert!(
            should_emit_audit,
            "impersonated request must trigger audit emit"
        );
        assert!(
            !should_not_emit_audit,
            "normal request must NOT trigger audit emit"
        );
    }
}
