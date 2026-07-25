// MFA self-service handlers
//
//   GET  /backoffice/mfa                  — enrollment status
//   POST /backoffice/mfa/enroll           — generate a secret (pending)
//   POST /backoffice/mfa/activate         — confirm with a code, get recovery codes
//   POST /backoffice/mfa/disable          — prove possession, clear MFA
//   POST /backoffice/mfa/recovery-codes   — prove possession, replace the set
//
// Self-service: every handler acts on the AUTHENTICATED operator
// (`ctx.user_id`) and never on an id from the path or body. There is no
// permission gate because there is nothing to gate — an operator managing
// their own second factor needs no platform privilege, and accepting a target
// id would turn these into account-takeover endpoints.

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use backoffice_identity::BackofficeUserId;

use crate::error::AppError;
use crate::middleware::auth::BackofficeUserContext;
use crate::state::BackofficeAppState;

/// Body for `POST /backoffice/mfa/enroll`.
#[derive(Debug, Deserialize)]
pub struct EnrollRequest {
    /// A code from the EXISTING authenticator, required only when MFA is
    /// already active. Re-enrolling is how an attacker holding a session token
    /// would swap in their own device, so the current factor must approve it.
    #[serde(default)]
    pub current_code: Option<String>,
}

/// Body for the operations that require proving possession.
#[derive(Debug, Deserialize)]
pub struct CodeRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct MfaStatusResponse {
    /// A secret exists — but see `active`.
    pub enrolled: bool,
    /// Enrollment was confirmed. This is the one that means "protected".
    pub active: bool,
    pub recovery_codes_remaining: usize,
}

#[derive(Debug, Serialize)]
pub struct EnrollResponse {
    /// For manual entry when a QR code cannot be scanned.
    pub secret: String,
    /// `otpauth://` URI to render as a QR code.
    pub provisioning_uri: String,
    /// Spelled out because a client that stops here leaves the operator
    /// believing they are protected when they are not.
    pub next_step: String,
}

/// Carries the one and only appearance of the plaintext recovery codes.
#[derive(Debug, Serialize)]
pub struct RecoveryCodesResponse {
    pub recovery_codes: Vec<String>,
    pub warning: String,
}

/// Text returned with every freshly issued set. Only hashes are stored, so
/// there is no endpoint that could show these again.
const RECOVERY_WARNING: &str = "Store these codes somewhere safe. They are shown once and cannot be retrieved again. \
     Each code works a single time.";

/// GET /backoffice/mfa — enrollment status for the authenticated operator.
pub async fn mfa_status_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
) -> Result<impl IntoResponse, Response> {
    let status = state
        .manage_mfa_use_case()
        .status(BackofficeUserId::from_uuid(ctx.user_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(MfaStatusResponse {
        enrolled: status.enrolled,
        active: status.active,
        recovery_codes_remaining: status.recovery_codes_remaining,
    }))
}

/// POST /backoffice/mfa/enroll — start enrollment, leaving it PENDING.
pub async fn enroll_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Json(body): Json<EnrollRequest>,
) -> Result<impl IntoResponse, Response> {
    let started = state
        .manage_mfa_use_case()
        .begin(
            BackofficeUserId::from_uuid(ctx.user_id),
            body.current_code.as_deref(),
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(EnrollResponse {
        secret: started.secret,
        provisioning_uri: started.provisioning_uri,
        next_step: "POST /backoffice/mfa/activate with a code from your authenticator. \
                    MFA is NOT active until you do."
            .to_string(),
    }))
}

/// POST /backoffice/mfa/activate — confirm enrollment, issue recovery codes.
pub async fn activate_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Json(body): Json<CodeRequest>,
) -> Result<impl IntoResponse, Response> {
    let issued = state
        .manage_mfa_use_case()
        .activate(BackofficeUserId::from_uuid(ctx.user_id), &body.code)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((
        StatusCode::OK,
        Json(RecoveryCodesResponse {
            recovery_codes: issued.recovery_codes,
            warning: RECOVERY_WARNING.to_string(),
        }),
    ))
}

/// POST /backoffice/mfa/disable — clear MFA, proving possession first.
pub async fn disable_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Json(body): Json<CodeRequest>,
) -> Result<impl IntoResponse, Response> {
    state
        .manage_mfa_use_case()
        .disable(BackofficeUserId::from_uuid(ctx.user_id), &body.code)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /backoffice/mfa/recovery-codes — replace the recovery code set.
pub async fn regenerate_recovery_codes_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Json(body): Json<CodeRequest>,
) -> Result<impl IntoResponse, Response> {
    let issued = state
        .manage_mfa_use_case()
        .regenerate_recovery_codes(BackofficeUserId::from_uuid(ctx.user_id), &body.code)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((
        StatusCode::OK,
        Json(RecoveryCodesResponse {
            recovery_codes: issued.recovery_codes,
            warning: RECOVERY_WARNING.to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `current_code` must be optional so a first enrollment needs no code —
    /// there is nothing to prove yet.
    #[test]
    fn enroll_request_accepts_an_empty_body() {
        let parsed: EnrollRequest = serde_json::from_str("{}").expect("empty body must parse");
        assert!(parsed.current_code.is_none());
    }

    #[test]
    fn enroll_request_accepts_a_current_code() {
        let parsed: EnrollRequest =
            serde_json::from_str(r#"{"current_code":"123456"}"#).expect("must parse");
        assert_eq!(parsed.current_code.as_deref(), Some("123456"));
    }

    /// The code is mandatory on every possession-proving operation.
    #[test]
    fn code_request_requires_the_code_field() {
        assert!(serde_json::from_str::<CodeRequest>("{}").is_err());
        assert!(serde_json::from_str::<CodeRequest>(r#"{"code":"123456"}"#).is_ok());
    }

    /// A client reading only `enrolled` would tell the operator they are
    /// protected while enrollment is still pending.
    #[test]
    fn status_response_separates_enrolled_from_active() {
        let pending = MfaStatusResponse {
            enrolled: true,
            active: false,
            recovery_codes_remaining: 0,
        };
        let json = serde_json::to_value(&pending).unwrap();
        assert_eq!(json["enrolled"], true);
        assert_eq!(json["active"], false);
    }

    #[test]
    fn recovery_response_carries_the_shown_once_warning() {
        let response = RecoveryCodesResponse {
            recovery_codes: vec!["ABCD-EFGH".to_string()],
            warning: RECOVERY_WARNING.to_string(),
        };
        let json = serde_json::to_value(&response).unwrap();
        assert!(json["warning"].as_str().unwrap().contains("shown once"));
        assert_eq!(json["recovery_codes"][0], "ABCD-EFGH");
    }
}
