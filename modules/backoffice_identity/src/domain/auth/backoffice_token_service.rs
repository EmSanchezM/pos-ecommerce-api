use common::BackofficeClaims;

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
}
