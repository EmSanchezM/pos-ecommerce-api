use serde::{Deserialize, Serialize};

/// Command DTO for backoffice login.
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthenticateBackofficeCommand {
    pub email: String,
    pub password: String,
}
