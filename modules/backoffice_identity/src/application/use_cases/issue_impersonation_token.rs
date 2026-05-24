// IssueImpersonationTokenUseCase
//
// Issues an impersonation token that lets a backoffice operator act as a tenant
// user in `api-gateway` for up to 15 minutes (NFR-SEC-4, FR-IMP-2).
//
// # Token shape (RFC 8693 delegation)
//
//   aud:          Tenant           (not Backoffice — api-gateway validates this)
//   sub:          tenant_user_id   (the impersonated user)
//   act.sub:      backoffice_user_id (the real actor)
//   act.sub_type: "backoffice_user"
//   act.email:    backoffice_user.email
//   exp:          iat + 900        (15 min, hardcoded constant — NFR-SEC-4)
//   signed with:  JWT_SECRET       (tenant secret — api-gateway can verify)
//
// # Why the token is signed with JWT_SECRET
//
// Per Decision 2 in sdd/backoffice-api/decisions: the simple pragmatic approach
// for v1 is `backoffice-api` holding both secrets.  The v2 migration target is an
// internal mTLS endpoint on `api-gateway`, but that is deferred.
//
// # Clean Architecture
//
// Token encoding lives in `JwtBackofficeTokenService::issue_impersonation_token`.
// This use case is the application-layer orchestration that:
//   1. Validates the 900s expiry constant is enforced.
//   2. Calls the domain service.
//   3. Returns the DTO.

use std::sync::Arc;

use uuid::Uuid;

use crate::application::dtos::ImpersonationTokenResponse;
use crate::domain::auth::BackofficeTokenService;
use crate::domain::entities::BackofficeUser;
use crate::error::BackofficeIdentityError;

/// Constant expiry for impersonation tokens — 15 minutes (NFR-SEC-4).
///
/// Any code path that issues with a longer duration is a security defect.
pub const IMPERSONATION_TOKEN_EXPIRY_SECONDS: i64 = 900;

pub struct IssueImpersonationTokenUseCase {
    token_service: Arc<dyn BackofficeTokenService>,
    tenant_secret: String,
}

impl IssueImpersonationTokenUseCase {
    /// Creates a new `IssueImpersonationTokenUseCase`.
    ///
    /// # Arguments
    ///
    /// * `token_service` — JWT service that implements `issue_impersonation_token`.
    /// * `tenant_secret` — The JWT_SECRET used to sign tenant tokens; api-gateway
    ///   validates with this secret.
    pub fn new(token_service: Arc<dyn BackofficeTokenService>, tenant_secret: String) -> Self {
        Self {
            token_service,
            tenant_secret,
        }
    }

    /// Issue an impersonation token for the given tenant user.
    ///
    /// The backoffice_user is the authenticated operator (from the request context).
    /// The tenant_user_id is the target being impersonated.
    pub async fn execute(
        &self,
        backoffice_user: &BackofficeUser,
        tenant_user_id: Uuid,
    ) -> Result<ImpersonationTokenResponse, BackofficeIdentityError> {
        let access_token = self.token_service.issue_impersonation_token(
            backoffice_user,
            tenant_user_id,
            &self.tenant_secret,
        )?;

        Ok(ImpersonationTokenResponse {
            access_token,
            expires_in: IMPERSONATION_TOKEN_EXPIRY_SECONDS,
        })
    }
}

// =============================================================================
// P5-T01 RED Tests — written before implementation
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Utc;
    use common::{ActorClaim, TokenAudience};
    use jsonwebtoken::{DecodingKey, Validation, decode};
    use serde::{Deserialize, Serialize};
    use uuid::{NoContext, Timestamp};

    use crate::domain::value_objects::{BackofficeEmail, BackofficeUserId};

    // -------------------------------------------------------------------------
    // Minimal token claims for decoding the impersonation token.
    // We use an ad-hoc struct so the test doesn't depend on TokenClaims from
    // identity (which carries many fields not needed here).
    // -------------------------------------------------------------------------

    #[derive(Debug, Serialize, Deserialize)]
    struct ImpersonationClaims {
        sub: Uuid,
        aud: TokenAudience,
        exp: i64,
        iat: i64,
        #[serde(default)]
        act: Option<ActorClaim>,
    }

    const TENANT_SECRET: &str = "tenant-secret-for-test-at-least-32-bytes";
    const BACKOFFICE_SECRET: &str = "backoffice-secret-for-test-at-least-32";

    fn make_backoffice_user() -> BackofficeUser {
        BackofficeUser::new(
            BackofficeUserId::new(),
            BackofficeEmail::new("operator@platform.com").unwrap(),
            "hashed_password".to_string(),
            None,
            true,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn make_use_case() -> IssueImpersonationTokenUseCase {
        use crate::infrastructure::JwtBackofficeTokenService;

        let svc = Arc::new(JwtBackofficeTokenService::with_issuer(
            BACKOFFICE_SECRET.to_string(),
            "backoffice-api:test".to_string(),
        ));

        IssueImpersonationTokenUseCase::new(svc, TENANT_SECRET.to_string())
    }

    fn decode_impersonation_token(token: &str) -> ImpersonationClaims {
        let mut validation = Validation::default();
        validation.validate_aud = false;
        validation.validate_exp = false; // avoid clock drift in tests

        decode::<ImpersonationClaims>(
            token,
            &DecodingKey::from_secret(TENANT_SECRET.as_bytes()),
            &validation,
        )
        .expect("token should decode with tenant secret")
        .claims
    }

    // -------------------------------------------------------------------------
    // P5-T01-a: issued token has aud: Tenant (not Backoffice)
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn issued_token_has_aud_tenant() {
        let uc = make_use_case();
        let user = make_backoffice_user();
        let tenant_user_id = Uuid::new_v7(Timestamp::now(NoContext));

        let resp = uc.execute(&user, tenant_user_id).await.unwrap();
        let claims = decode_impersonation_token(&resp.access_token);

        assert_eq!(claims.aud, TokenAudience::Tenant);
    }

    // -------------------------------------------------------------------------
    // P5-T01-b: sub = tenant_user_id (the impersonated user)
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn issued_token_sub_equals_tenant_user_id() {
        let uc = make_use_case();
        let user = make_backoffice_user();
        let tenant_user_id = Uuid::new_v7(Timestamp::now(NoContext));

        let resp = uc.execute(&user, tenant_user_id).await.unwrap();
        let claims = decode_impersonation_token(&resp.access_token);

        assert_eq!(claims.sub, tenant_user_id);
    }

    // -------------------------------------------------------------------------
    // P5-T01-c: act.sub = backoffice_user_id (the real actor)
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn issued_token_act_sub_equals_backoffice_user_id() {
        let uc = make_use_case();
        let user = make_backoffice_user();
        let tenant_user_id = Uuid::new_v7(Timestamp::now(NoContext));

        let resp = uc.execute(&user, tenant_user_id).await.unwrap();
        let claims = decode_impersonation_token(&resp.access_token);

        let act = claims.act.expect("act claim must be present");
        assert_eq!(act.sub, *user.id().as_uuid());
    }

    // -------------------------------------------------------------------------
    // P5-T01-d: act.sub_type = "backoffice_user"
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn issued_token_act_sub_type_is_backoffice_user() {
        let uc = make_use_case();
        let user = make_backoffice_user();
        let tenant_user_id = Uuid::new_v7(Timestamp::now(NoContext));

        let resp = uc.execute(&user, tenant_user_id).await.unwrap();
        let claims = decode_impersonation_token(&resp.access_token);

        let act = claims.act.expect("act claim must be present");
        assert_eq!(act.sub_type, "backoffice_user");
    }

    // -------------------------------------------------------------------------
    // P5-T01-e: exp - iat <= 900 (15-min cap, NFR-SEC-4)
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn issued_token_expiry_is_at_most_900_seconds() {
        let uc = make_use_case();
        let user = make_backoffice_user();
        let tenant_user_id = Uuid::new_v7(Timestamp::now(NoContext));

        let resp = uc.execute(&user, tenant_user_id).await.unwrap();
        let claims = decode_impersonation_token(&resp.access_token);

        assert!(
            claims.exp - claims.iat <= IMPERSONATION_TOKEN_EXPIRY_SECONDS,
            "exp - iat was {} but must be <= 900",
            claims.exp - claims.iat
        );
    }

    // -------------------------------------------------------------------------
    // P5-T01-f: expires_in in the DTO equals 900
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn response_expires_in_is_900() {
        let uc = make_use_case();
        let user = make_backoffice_user();
        let tenant_user_id = Uuid::new_v7(Timestamp::now(NoContext));

        let resp = uc.execute(&user, tenant_user_id).await.unwrap();

        assert_eq!(resp.expires_in, IMPERSONATION_TOKEN_EXPIRY_SECONDS);
    }

    // -------------------------------------------------------------------------
    // P5-T01-g: token is NOT decodable with the backoffice secret
    //
    // Ensures the impersonation token is signed with the TENANT secret,
    // not the backoffice secret (important isolation property).
    // -------------------------------------------------------------------------
    #[test]
    fn impersonation_token_not_decodable_with_backoffice_secret() {
        use crate::infrastructure::JwtBackofficeTokenService;
        use jsonwebtoken::{DecodingKey, Validation, decode};

        let svc = JwtBackofficeTokenService::with_issuer(
            BACKOFFICE_SECRET.to_string(),
            "backoffice-api:test".to_string(),
        );

        let user = make_backoffice_user();
        let tenant_user_id = Uuid::new_v7(Timestamp::now(NoContext));

        let token = svc
            .issue_impersonation_token(&user, tenant_user_id, TENANT_SECRET)
            .expect("should issue token");

        // Attempt to decode with the WRONG (backoffice) secret — must fail.
        let mut validation = Validation::default();
        validation.validate_aud = false;
        let result = decode::<ImpersonationClaims>(
            &token,
            &DecodingKey::from_secret(BACKOFFICE_SECRET.as_bytes()),
            &validation,
        );
        assert!(
            result.is_err(),
            "impersonation token must NOT decode with backoffice secret"
        );
    }
}
