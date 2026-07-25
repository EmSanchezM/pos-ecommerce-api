use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::mfa::{
    activate_handler, disable_handler, enroll_handler, mfa_status_handler,
    regenerate_recovery_codes_handler,
};
use crate::state::BackofficeAppState;

/// Router for self-service MFA management.
///
/// Mounted behind the backoffice auth middleware — every handler acts on the
/// authenticated operator, so authentication IS the authorization here. There
/// is deliberately no route that takes a target user id: that would turn
/// enrollment management into an account-takeover surface.
pub fn mfa_router(state: BackofficeAppState) -> Router {
    Router::new()
        .route("/", get(mfa_status_handler))
        .route("/enroll", post(enroll_handler))
        .route("/activate", post(activate_handler))
        .route("/disable", post(disable_handler))
        .route("/recovery-codes", post(regenerate_recovery_codes_handler))
        .with_state(state)
}
