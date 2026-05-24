use chrono::{Duration, Utc};
use common::{BackofficeClaims, TokenAudience};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::domain::auth::BackofficeTokenService;
use crate::domain::entities::BackofficeUser;
use crate::error::BackofficeIdentityError;

/// Access token duration for backoffice tokens: 7 days.
///
/// Backoffice operators are internal users; a longer session is acceptable
/// since the audience isolation and secret separation are the primary controls.
const BACKOFFICE_ACCESS_TOKEN_DURATION_SECS: i64 = 7 * 24 * 3600;

pub struct JwtBackofficeTokenService {
    secret: String,
    issuer: String,
}

impl JwtBackofficeTokenService {
    /// Creates a new `JwtBackofficeTokenService`.
    ///
    /// Reads `JWT_BACKOFFICE_ISSUER` from env; defaults to `"backoffice-api"`.
    pub fn new(secret: String) -> Self {
        let issuer = std::env::var("JWT_BACKOFFICE_ISSUER")
            .unwrap_or_else(|_| "backoffice-api".to_string());
        Self { secret, issuer }
    }

    /// Creates with explicit issuer (useful for tests).
    pub fn with_issuer(secret: String, issuer: String) -> Self {
        Self { secret, issuer }
    }
}

impl BackofficeTokenService for JwtBackofficeTokenService {
    fn issue_backoffice_token(
        &self,
        user: &BackofficeUser,
        permissions: &[String],
    ) -> Result<String, BackofficeIdentityError> {
        let now = Utc::now();
        let exp = now + Duration::seconds(BACKOFFICE_ACCESS_TOKEN_DURATION_SECS);

        let claims = BackofficeClaims {
            sub: *user.id().as_uuid(),
            aud: TokenAudience::Backoffice,
            iss: self.issuer.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            permissions: permissions.to_vec(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| BackofficeIdentityError::JwtError(e.to_string()))
    }

    fn validate_backoffice_token(
        &self,
        token: &str,
    ) -> Result<BackofficeClaims, BackofficeIdentityError> {
        let mut validation = Validation::default();
        // Check aud manually so we can return the right error variant.
        validation.validate_aud = false;

        let claims = decode::<BackofficeClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                BackofficeIdentityError::TokenExpired
            }
            _ => BackofficeIdentityError::InvalidToken,
        })?;

        if claims.aud != TokenAudience::Backoffice {
            return Err(BackofficeIdentityError::InvalidToken);
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{BackofficeEmail, BackofficeUserId};
    use chrono::Utc;

    fn make_user() -> BackofficeUser {
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

    fn make_service() -> JwtBackofficeTokenService {
        JwtBackofficeTokenService::with_issuer(
            "backoffice-secret-at-least-32-bytes-long".to_string(),
            "backoffice-api:test".to_string(),
        )
    }

    #[test]
    fn issue_backoffice_token_produces_valid_jwt() {
        let svc = make_service();
        let user = make_user();
        let token = svc
            .issue_backoffice_token(&user, &["platform:org.list".to_string()])
            .unwrap();
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn issued_token_has_aud_backoffice() {
        let svc = make_service();
        let user = make_user();
        let token = svc
            .issue_backoffice_token(&user, &["platform:org.list".to_string()])
            .unwrap();
        let claims = svc.validate_backoffice_token(&token).unwrap();
        assert_eq!(claims.aud, TokenAudience::Backoffice);
    }

    #[test]
    fn issued_token_iss_matches_configured_issuer() {
        let svc = make_service();
        let user = make_user();
        let token = svc.issue_backoffice_token(&user, &[]).unwrap();
        let claims = svc.validate_backoffice_token(&token).unwrap();
        assert_eq!(claims.iss, "backoffice-api:test");
    }

    #[test]
    fn issued_token_sub_equals_user_id() {
        let svc = make_service();
        let user = make_user();
        let token = svc.issue_backoffice_token(&user, &[]).unwrap();
        let claims = svc.validate_backoffice_token(&token).unwrap();
        assert_eq!(claims.sub, *user.id().as_uuid());
    }

    #[test]
    fn issued_token_carries_permissions() {
        let svc = make_service();
        let user = make_user();
        let perms = vec![
            "platform:org.list".to_string(),
            "platform:org.suspend".to_string(),
        ];
        let token = svc.issue_backoffice_token(&user, &perms).unwrap();
        let claims = svc.validate_backoffice_token(&token).unwrap();
        assert_eq!(claims.permissions, perms);
    }

    #[test]
    fn validate_rejects_token_signed_by_different_secret() {
        let svc = make_service();
        let other_svc = JwtBackofficeTokenService::with_issuer(
            "completely-different-secret-32-bytes-long".to_string(),
            "backoffice-api:test".to_string(),
        );
        let user = make_user();
        let token = other_svc.issue_backoffice_token(&user, &[]).unwrap();
        let result = svc.validate_backoffice_token(&token);
        assert!(matches!(result, Err(BackofficeIdentityError::InvalidToken)));
    }

    #[test]
    fn validate_rejects_tenant_audience_token() {
        // Manually craft a token with aud: Tenant using the same secret.
        // This simulates cross-audience token smuggling.
        use jsonwebtoken::{EncodingKey, Header, encode};
        use uuid::{NoContext, Timestamp};

        let secret = "backoffice-secret-at-least-32-bytes-long";
        let svc = JwtBackofficeTokenService::with_issuer(
            secret.to_string(),
            "backoffice-api:test".to_string(),
        );

        let now = Utc::now();
        let tenant_claims = BackofficeClaims {
            sub: uuid::Uuid::new_v7(Timestamp::now(NoContext)),
            aud: TokenAudience::Tenant, // wrong audience
            iss: "api-gateway:test".to_string(),
            exp: (now + Duration::hours(1)).timestamp(),
            iat: now.timestamp(),
            permissions: vec![],
        };

        let token = encode(
            &Header::default(),
            &tenant_claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let result = svc.validate_backoffice_token(&token);
        assert!(
            matches!(result, Err(BackofficeIdentityError::InvalidToken)),
            "Expected InvalidToken for aud:Tenant presented to backoffice validator"
        );
    }

    #[test]
    fn validate_rejects_expired_token() {
        use jsonwebtoken::{EncodingKey, Header, encode};
        use uuid::{NoContext, Timestamp};

        let secret = "backoffice-secret-at-least-32-bytes-long";
        let svc = JwtBackofficeTokenService::with_issuer(
            secret.to_string(),
            "backoffice-api:test".to_string(),
        );

        let now = Utc::now();
        let expired_claims = BackofficeClaims {
            sub: uuid::Uuid::new_v7(Timestamp::now(NoContext)),
            aud: TokenAudience::Backoffice,
            iss: "backoffice-api:test".to_string(),
            exp: (now - Duration::hours(1)).timestamp(), // already expired
            iat: (now - Duration::hours(2)).timestamp(),
            permissions: vec![],
        };

        let token = encode(
            &Header::default(),
            &expired_claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let result = svc.validate_backoffice_token(&token);
        assert!(matches!(
            result,
            Err(BackofficeIdentityError::TokenExpired)
        ));
    }
}
