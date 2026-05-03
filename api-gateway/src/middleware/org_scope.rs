//! Tenancy v1.1 enforcement helpers — used by handlers to make sure a
//! request from user A in org X cannot reach data of org Y.
//!
//! Three primitives:
//!
//! - [`require_org_match`] — verifies a target `organization_id` from a path
//!   parameter belongs to the user (super_admin bypasses).
//! - [`verify_store_in_org`] — runs `SELECT 1 FROM stores WHERE id = $1 AND
//!   organization_id = $2` so that store ids coming off `?store_id=` query
//!   params can't be cross-org. One extra query per touched store; v1.2
//!   caches by id with a short TTL.
//! - [`require_feature`] — loads the active org's plan, returns 403 if the
//!   `feature_flags[name]` flag is off. Lets us gate entire route trees by
//!   plan tier without touching individual handlers.
//!
//! Every helper returns a fully-built [`Response`] on failure so the call
//! site stays a one-liner: `verify_store_in_org(...).await?;`. None of them
//! short-circuits on missing JWT scope — that's the auth middleware's job.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use identity::{ErrorResponse, UserContext};
use sqlx::PgPool;
use uuid::Uuid;

/// Verifies the `target_org_id` belongs to the user's organization.
///
/// `super_admin` bypasses the check (they manage every tenant). Any other
/// caller must have a `Some(organization_id)` claim that matches the target,
/// otherwise this returns 403 with `CROSS_ORG_ACCESS_DENIED`.
#[allow(clippy::result_large_err)]
pub fn require_org_match(ctx: &UserContext, target_org_id: Uuid) -> Result<(), Response> {
    if ctx.is_super_admin() {
        return Ok(());
    }
    match ctx.organization_id() {
        Some(my_org_id) if my_org_id == target_org_id => Ok(()),
        _ => Err(forbidden(
            "CROSS_ORG_ACCESS_DENIED",
            "Access denied: target resource belongs to a different organization",
        )),
    }
}

/// Confirms `store_id` belongs to the user's organization. `super_admin`
/// bypasses (their queries can hop tenants). Anyone else triggers a single
/// `SELECT 1 FROM stores WHERE id = $1 AND organization_id = $2`. v1.2 caches
/// the lookup with a short TTL to amortise the query.
///
/// Returns 403 with `CROSS_ORG_STORE_DENIED` on miss, 500 if the DB errors,
/// and `STORE_NOT_FOUND` (404) if the store id doesn't exist at all.
pub async fn verify_store_in_org(
    pool: &PgPool,
    ctx: &UserContext,
    store_id: Uuid,
) -> Result<(), Response> {
    if ctx.is_super_admin() {
        return Ok(());
    }
    let Some(my_org_id) = ctx.organization_id() else {
        return Err(forbidden(
            "ORG_SCOPE_MISSING",
            "Token does not carry an organization scope; re-login required",
        ));
    };

    let row: Option<(Option<Uuid>,)> =
        sqlx::query_as("SELECT organization_id FROM stores WHERE id = $1")
            .bind(store_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "verify_store_in_org SELECT failed");
                internal_error()
            })?;

    let Some((store_org,)) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "STORE_NOT_FOUND",
                format!("Store not found: {}", store_id),
            )),
        )
            .into_response());
    };
    match store_org {
        Some(org) if org == my_org_id => Ok(()),
        _ => Err(forbidden(
            "CROSS_ORG_STORE_DENIED",
            "Store belongs to a different organization",
        )),
    }
}

/// Returns 403 unless the user's organization plan has `feature` enabled.
/// Looks up `organization_plans.feature_flags[feature]` (a JSONB bool) for
/// the user's org. `super_admin` always passes.
///
/// Treats a missing plan / missing key / non-bool value as "feature off"
/// (fail-closed).
pub async fn require_feature(
    pool: &PgPool,
    ctx: &UserContext,
    feature: &str,
) -> Result<(), Response> {
    if ctx.is_super_admin() {
        return Ok(());
    }
    let Some(org_id) = ctx.organization_id() else {
        return Err(forbidden(
            "ORG_SCOPE_MISSING",
            "Token does not carry an organization scope; re-login required",
        ));
    };

    // Pull just the boolean we care about so the query stays cheap.
    let row: Option<(Option<bool>,)> = sqlx::query_as(
        r#"
        SELECT (feature_flags ->> $2)::boolean
        FROM organization_plans
        WHERE organization_id = $1
        "#,
    )
    .bind(org_id)
    .bind(feature)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "require_feature SELECT failed");
        internal_error()
    })?;

    match row {
        Some((Some(true),)) => Ok(()),
        _ => Err(forbidden(
            "FEATURE_DISABLED",
            &format!(
                "Feature `{}` is not enabled for the current organization plan",
                feature
            ),
        )),
    }
}

#[allow(clippy::result_large_err)]
fn forbidden(code: &str, message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse::new(code, message)),
    )
        .into_response()
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::internal_error()),
    )
        .into_response()
}
