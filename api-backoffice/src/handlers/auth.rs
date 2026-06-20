// Backoffice auth handlers
//
// POST /backoffice/auth/login — authenticate a backoffice user and return a JWT.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use backoffice_identity::AuthenticateBackofficeCommand;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::BackofficeAppState;

/// Request body for `POST /backoffice/auth/login`.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Response body for a successful login.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub user_id: String,
    pub email: String,
}

/// POST /backoffice/auth/login
///
/// Authenticates a backoffice user with email + password.
/// Returns a `Backoffice`-audience JWT on success.
/// Returns 401 on invalid credentials or inactive account.
pub async fn login_handler(
    State(state): State<BackofficeAppState>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let cmd = AuthenticateBackofficeCommand {
        email: body.email,
        password: body.password,
    };

    let response = state
        .authenticate_use_case()
        .execute(cmd)
        .await
        .map_err(AppError::from)?;

    let login_response = LoginResponse {
        access_token: response.access_token,
        expires_in: response.expires_in,
        user_id: response.user_id.to_string(),
        email: response.email,
    };

    Ok((StatusCode::OK, Json(login_response)))
}
