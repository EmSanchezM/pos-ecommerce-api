use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response DTO returned after a successful backoffice login.
#[derive(Debug, Serialize, Deserialize)]
pub struct BackofficeAuthResponse {
    pub access_token: String,
    /// Token lifetime in seconds.
    pub expires_in: i64,
    pub user_id: Uuid,
    pub email: String,
}

/// Response DTO returned after issuing an impersonation token.
///
/// The `access_token` is a JWT with `aud: Tenant`, `act` claim, signed with
/// `JWT_SECRET`. The frontend should decode it, detect the `act` claim, and
/// render the impersonation banner (FR-IMP-6).
#[derive(Debug, Serialize, Deserialize)]
pub struct ImpersonationTokenResponse {
    pub access_token: String,
    /// Always 900 (15 minutes) — hardcoded per NFR-SEC-4.
    pub expires_in: i64,
}
