// Subscriptions routes — SaaS billing of the platform itself.
//
// Public:
//   GET  /api/v1/public/subscription-plans          — list active plans
//   GET  /api/v1/public/subscription-plans/{id}     — single plan lookup
//
// Authenticated:
//   GET    /api/v1/subscription-plans               — list active (auth)
//   GET    /api/v1/subscription-plans/admin         — paginated, all plans
//   GET    /api/v1/subscription-plans/{id}          — single plan (auth)
//   POST   /api/v1/subscription-plans               — create plan
//   PUT    /api/v1/subscription-plans/{id}          — update plan
//   DELETE /api/v1/subscription-plans/{id}          — soft-deactivate plan
//
//   GET    /api/v1/admin/subscriptions              — admin cross-org list
//
//   POST   /api/v1/organizations/{org_id}/subscription           — subscribe
//   GET    /api/v1/organizations/{org_id}/subscription           — read
//   GET    /api/v1/organizations/{org_id}/subscription/cycles    — history
//   POST   /api/v1/organizations/{org_id}/subscription/cancel
//   POST   /api/v1/organizations/{org_id}/subscription/resume
//   POST   /api/v1/organizations/{org_id}/subscription/change-plan

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handlers::subscriptions::{
    cancel_subscription_handler, change_plan_handler, create_plan_handler, deactivate_plan_handler,
    get_plan_handler, get_subscription_handler, list_billing_cycles_handler, list_plans_handler,
    list_plans_paginated_handler, list_subscriptions_admin_handler, public_get_plan_handler,
    public_list_plans_handler, resume_subscription_handler, subscribe_organization_handler,
    update_plan_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn subscription_plans_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_plans_handler).post(create_plan_handler))
        .route("/admin", get(list_plans_paginated_handler))
        .route(
            "/{id}",
            get(get_plan_handler)
                .put(update_plan_handler)
                .delete(deactivate_plan_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// PUBLIC: no auth. Mounted under `/api/v1/public/subscription-plans`.
pub fn public_subscription_plans_router() -> Router<AppState> {
    Router::new()
        .route("/", get(public_list_plans_handler))
        .route("/{id}", get(public_get_plan_handler))
}

/// `/api/v1/organizations/{org_id}/subscription/*`. Mounted with the org id
/// as a path prefix so the handler-level `require_org_match` check has
/// something to verify against.
pub fn organization_subscription_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/{org_id}/subscription",
            post(subscribe_organization_handler).get(get_subscription_handler),
        )
        .route(
            "/{org_id}/subscription/cycles",
            get(list_billing_cycles_handler),
        )
        .route(
            "/{org_id}/subscription/cancel",
            post(cancel_subscription_handler),
        )
        .route(
            "/{org_id}/subscription/resume",
            post(resume_subscription_handler),
        )
        .route(
            "/{org_id}/subscription/change-plan",
            post(change_plan_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn admin_subscriptions_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_subscriptions_admin_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
