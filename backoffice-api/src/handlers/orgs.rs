// Organization admin handlers
//
// GET /backoffice/orgs — stub (returns 501) until Phase 6 full implementation.

use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::error::ErrorResponse;

/// GET /backoffice/orgs — stub handler (Phase 3 placeholder, Phase 6 full impl).
///
/// Returns 501 Not Implemented with a JSON error body.
pub async fn list_orgs_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse::not_implemented()),
    )
}
