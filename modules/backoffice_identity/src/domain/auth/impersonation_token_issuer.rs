use async_trait::async_trait;
use uuid::Uuid;

use crate::error::BackofficeIdentityError;

/// Domain port — mints a tenant-audience impersonation token (with the RFC 8693
/// `act` claim) for `tenant_user_id` on behalf of a backoffice operator.
///
/// v2: the backoffice no longer signs tenant tokens itself. The infrastructure
/// implementation calls the api-gateway internal endpoint, so the tenant
/// signing key (JWT_SECRET) never lives in the backoffice process. The call is
/// async (it crosses a service boundary), unlike the old local-signing method.
#[async_trait]
pub trait ImpersonationTokenIssuer: Send + Sync {
    /// Returns the signed impersonation JWT (aud: Tenant, 15-min expiry).
    async fn issue_impersonation_token(
        &self,
        tenant_user_id: Uuid,
        operator_id: Uuid,
        operator_email: &str,
    ) -> Result<String, BackofficeIdentityError>;
}
