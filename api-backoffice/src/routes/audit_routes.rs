use axum::{Router, routing::get};

use crate::handlers::audit_log::list_audit_log_handler;
use crate::state::BackofficeAppState;

/// Router for reading the backoffice audit log.
///
/// Requires the backoffice auth middleware; the handler enforces
/// `platform:audit.read`. Read-only by design — the log is append-only and
/// this router must never gain a mutating route (FR-AUD-4, NFR-SEC-2).
pub fn audit_router(state: BackofficeAppState) -> Router {
    Router::new()
        .route("/", get(list_audit_log_handler))
        .with_state(state)
}
