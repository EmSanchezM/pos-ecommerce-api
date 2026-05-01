// Analytics routes: dashboards, widgets, KPIs, and reports.
//
// Mounted at /api/v1/analytics. All endpoints require authentication.
// Read endpoints additionally require `reports:analytics`; write endpoints
// require `analytics:dashboards:write`. Permission enforcement happens
// inside each handler via `require_permission`.

use axum::{
    Router, middleware,
    routing::{delete, get, post},
};

use crate::handlers::analytics::{
    add_widget_handler, create_dashboard_handler, get_dashboard_overview_handler,
    get_kpi_snapshot_handler, list_dashboards_handler, remove_widget_handler, run_report_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

/// Builds the `/api/v1/analytics` router.
pub fn analytics_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/dashboards",
            get(list_dashboards_handler).post(create_dashboard_handler),
        )
        .route(
            "/dashboards/{id}/overview",
            get(get_dashboard_overview_handler),
        )
        .route("/dashboards/{id}/widgets", post(add_widget_handler))
        .route("/widgets/{id}", delete(remove_widget_handler))
        .route("/kpis/{kpi_key}", get(get_kpi_snapshot_handler))
        .route("/reports/run", post(run_report_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
