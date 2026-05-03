// Tenancy routes: organizations + sub-resources (plan, domains, branding) +
// public lookup. v1.0 keeps every authenticated route super_admin-only.
//
// /api/v1/organizations                  - org CRUD + suspend/activate
// /api/v1/organizations/{id}/plan        - tier + feature flags + limits
// /api/v1/organizations/{id}/domains     - custom domain CRUD + verify + primary
// /api/v1/organizations/{id}/branding    - logo + colors + theme + custom CSS
// /api/v1/public/organizations/by-slug/{slug}    - storefront lookup
// /api/v1/public/organizations/by-domain/{host}  - storefront lookup

use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};

use crate::handlers::tenancy::{
    activate_organization_handler, delete_domain_handler, get_branding_handler,
    get_organization_handler, get_plan_handler, list_domains_handler, list_organizations_handler,
    public_org_by_domain_handler, public_org_by_slug_handler, register_domain_handler,
    register_organization_handler, set_feature_flag_handler, set_plan_handler,
    set_primary_domain_handler, suspend_organization_handler, update_organization_handler,
    upsert_branding_handler, verify_domain_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn tenancy_organizations_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_organizations_handler).post(register_organization_handler),
        )
        .route(
            "/{id}",
            get(get_organization_handler).put(update_organization_handler),
        )
        .route("/{id}/suspend", post(suspend_organization_handler))
        .route("/{id}/activate", post(activate_organization_handler))
        // Plan
        .route("/{id}/plan", get(get_plan_handler).put(set_plan_handler))
        .route("/{id}/plan/feature", put(set_feature_flag_handler))
        // Domains
        .route(
            "/{id}/domains",
            get(list_domains_handler).post(register_domain_handler),
        )
        .route("/{id}/domains/{did}", delete(delete_domain_handler))
        .route("/{id}/domains/{did}/verify", post(verify_domain_handler))
        .route(
            "/{id}/domains/{did}/set-primary",
            post(set_primary_domain_handler),
        )
        // Branding
        .route(
            "/{id}/branding",
            get(get_branding_handler).put(upsert_branding_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// PUBLIC: no auth. Mounted under `/api/v1/public/organizations`.
pub fn public_tenancy_router() -> Router<AppState> {
    Router::new()
        .route("/by-slug/{slug}", get(public_org_by_slug_handler))
        .route("/by-domain/{host}", get(public_org_by_domain_handler))
}
