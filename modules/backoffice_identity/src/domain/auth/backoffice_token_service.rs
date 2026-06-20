use common::BackofficeClaims;
use uuid::Uuid;

use crate::domain::entities::BackofficeUser;
use crate::error::BackofficeIdentityError;

/// Domain port — backoffice JWT issuance and validation.
///
/// Implemented by `JwtBackofficeTokenService` in the infrastructure layer.
pub trait BackofficeTokenService: Send + Sync {
    /// Issues a backoffice access token (aud: Backoffice).
    fn issue_backoffice_token(
        &self,
        user: &BackofficeUser,
        permissions: &[String],
    ) -> Result<String, BackofficeIdentityError>;

    /// Validates a backoffice token and returns its claims.
    ///
    /// Returns `Err(InvalidToken)` for any token with aud != Backoffice.
    fn validate_backoffice_token(
        &self,
        token: &str,
    ) -> Result<BackofficeClaims, BackofficeIdentityError>;

    /// Issues an impersonation token (aud: Tenant, 15-min expiry, act claim).
    ///
    /// The token is signed with `tenant_secret` (JWT_SECRET) so that `api-gateway`
    /// can validate it with its own secret (Decision 2, sdd/backoffice-api/decisions).
    ///
    /// # Claims set
    /// - `aud: Tenant`
    /// - `sub: tenant_user_id`
    /// - `act.sub: backoffice_user.id`
    /// - `act.sub_type: "backoffice_user"`
    /// - `act.email: backoffice_user.email`
    /// - `exp: iat + 900` (NFR-SEC-4)
    fn issue_impersonation_token(
        &self,
        backoffice_user: &BackofficeUser,
        tenant_user_id: Uuid,
        tenant_secret: &str,
    ) -> Result<String, BackofficeIdentityError>;
}
