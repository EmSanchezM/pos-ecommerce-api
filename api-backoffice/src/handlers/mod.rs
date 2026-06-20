pub mod analytics;
pub mod auth;
pub mod dunning;
pub mod health;
pub mod impersonate;
pub mod orgs;
pub mod plans;
pub mod subscriptions;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::error::ErrorResponse;

/// Returns 400 `REASON_REQUIRED` if `reason` is blank — enforced before any
/// state change (NFR-SEC-5). Shared by every state-mutating backoffice handler.
#[allow(clippy::result_large_err)]
pub(crate) fn reason_guard(reason: &str) -> Result<(), Response> {
    if reason.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "REASON_REQUIRED",
                "The 'reason' field is required for all state-mutating backoffice actions",
            )),
        )
            .into_response());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::reason_guard;

    #[test]
    fn empty_reason_is_rejected() {
        assert!(reason_guard("").is_err());
        assert!(reason_guard("   \t\n").is_err());
    }

    #[test]
    fn non_empty_reason_passes() {
        assert!(reason_guard("price correction for 2026 catalogue").is_ok());
    }
}
