use axum::{Router, routing::post};

use crate::handlers::dunning::trigger_dunning_handler;
use crate::state::BackofficeAppState;

/// Router for backoffice manual-dunning endpoints.
///
/// Requires the backoffice auth middleware; the handler enforces
/// `platform:dunning.trigger`.
pub fn dunning_router(state: BackofficeAppState) -> Router {
    Router::new()
        .route("/{attempt_id}/trigger", post(trigger_dunning_handler))
        .with_state(state)
}
