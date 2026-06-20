use axum::{Router, routing::get};

use crate::handlers::analytics::{get_kpi_handler, overview_handler};
use crate::state::BackofficeAppState;

/// Router for backoffice cross-org analytics endpoints.
///
/// Requires the backoffice auth middleware; each handler enforces
/// `platform:analytics.read`.
pub fn analytics_router(state: BackofficeAppState) -> Router {
    Router::new()
        .route("/overview", get(overview_handler))
        .route("/kpis/{kpi_key}", get(get_kpi_handler))
        .with_state(state)
}
