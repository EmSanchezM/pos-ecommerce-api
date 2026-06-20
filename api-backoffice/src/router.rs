// Backoffice router assembly
//
// Mounts all backoffice routes. The auth middleware is applied to
// authenticated routes only; /health and /backoffice/auth/login are public.

use axum::{Router, middleware, routing::get};

use crate::handlers::health::health_handler;
use crate::middleware::auth::backoffice_auth_middleware;
use crate::routes::{
    analytics_router, auth_router, dunning_router, impersonate_router, org_router, plan_router,
    subscription_router,
};
use crate::state::BackofficeAppState;

/// Build the complete backoffice API router.
///
/// Route layout:
/// - `GET /health`                                — public, no auth
/// - `POST /backoffice/auth/login`                — public, no auth (FR-END-7)
/// - `GET /backoffice/orgs`                       — requires aud:Backoffice JWT
/// - `POST /backoffice/orgs/{id}/suspend`         — requires platform:org.suspend
/// - `POST /backoffice/impersonate/{user_id}`     — requires platform:user.impersonate
/// - `GET/POST /backoffice/plans`                 — requires platform:plan.read/create
/// - `PUT /backoffice/plans/{id}`                 — requires platform:plan.update
/// - `POST /backoffice/plans/{id}/deactivate`     — requires platform:plan.update
/// - `GET /backoffice/subscriptions/{org_id}`     — requires platform:org.list
/// - `POST /backoffice/subscriptions/{org_id}/force-cancel` — platform:subscription.force_cancel
/// - `POST /backoffice/subscriptions/{org_id}/change-plan`  — platform:subscription.override_billing
/// - `POST /backoffice/subscriptions/{org_id}/resume`       — platform:subscription.override_billing
/// - `POST /backoffice/dunning/{attempt_id}/trigger`        — platform:dunning.trigger
/// - `GET /backoffice/analytics/overview`                   — platform:analytics.read
/// - `GET /backoffice/analytics/kpis/{kpi_key}`             — platform:analytics.read
pub fn build_router(state: BackofficeAppState) -> Router {
    // Public routes — no auth middleware
    let public_routes = Router::new()
        .route("/health", get(health_handler))
        .nest("/backoffice/auth", auth_router(state.clone()));

    // Authenticated routes — backoffice JWT middleware applied
    let authenticated_routes = Router::new()
        .nest("/backoffice/orgs", org_router(state.clone()))
        .nest("/backoffice/impersonate", impersonate_router(state.clone()))
        .nest("/backoffice/plans", plan_router(state.clone()))
        .nest(
            "/backoffice/subscriptions",
            subscription_router(state.clone()),
        )
        .nest("/backoffice/dunning", dunning_router(state.clone()))
        .nest("/backoffice/analytics", analytics_router(state.clone()))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            backoffice_auth_middleware,
        ));

    public_routes.merge(authenticated_routes)
}
