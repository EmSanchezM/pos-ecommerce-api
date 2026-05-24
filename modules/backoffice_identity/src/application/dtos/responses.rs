use uuid::Uuid;

/// Response DTO returned after a successful backoffice login.
#[derive(Debug)]
pub struct BackofficeAuthResponse {
    pub access_token: String,
    /// Token lifetime in seconds.
    pub expires_in: i64,
    pub user_id: Uuid,
    pub email: String,
}
