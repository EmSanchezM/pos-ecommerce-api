//! Tenancy enforcement helpers — used by handlers to make sure a request
//! from user A in org X cannot reach data of org Y.
//!
//! Three primitives:
//!
//! - [`require_org_match`] — verifies a target `organization_id` from a path
//!   parameter belongs to the user (super_admin bypasses).
//! - [`verify_store_in_org`] — looks up a store's `organization_id` so that
//!   store ids coming off `?store_id=` query params can't be cross-org.
//!   v1.2 caches the lookup process-wide with a 60s TTL to amortise the
//!   query across the rollout to ~14 modules.
//! - [`require_feature`] — loads the active org's plan, returns 403 if the
//!   `feature_flags[name]` flag is off. Lets us gate entire route trees by
//!   plan tier without touching individual handlers.
//!
//! Every helper returns a fully-built [`Response`] on failure so the call
//! site stays a one-liner: `verify_store_in_org(...).await?;`. None of them
//! short-circuits on missing JWT scope — that's the auth middleware's job.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use identity::{ErrorResponse, UserContext};
use sqlx::PgPool;
use uuid::Uuid;

/// TTL for cached `(store_id → organization_id)` lookups. Short on purpose:
/// store mutations (rare) become visible within at most a minute without an
/// explicit invalidation hook. v1.3 will add explicit invalidation when a
/// store moves between orgs (also rare).
const STORE_ORG_CACHE_TTL: Duration = Duration::from_secs(60);

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
/// bypasses (their queries can hop tenants).
///
/// Backed by a process-wide TTL cache (see [`STORE_ORG_CACHE_TTL`]). Cold
/// path: a single `SELECT organization_id FROM stores WHERE id = $1`. Warm
/// path: zero queries. Both hits and misses (including non-existent ids) are
/// cached so a stuck client hammering the same bad id doesn't stampede.
///
/// Returns 403 with `CROSS_ORG_STORE_DENIED` on org mismatch, 500 if the DB
/// errors, and `STORE_NOT_FOUND` (404) if the store id doesn't exist at all.
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

    let result = match store_org_cache().get(store_id) {
        Some(cached) => cached,
        None => {
            let row: Option<(Option<Uuid>,)> =
                sqlx::query_as("SELECT organization_id FROM stores WHERE id = $1")
                    .bind(store_id)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| {
                        tracing::error!(error = %e, "verify_store_in_org SELECT failed");
                        internal_error()
                    })?;
            let resolved = row.map(|(org,)| org);
            store_org_cache().put(store_id, resolved);
            resolved
        }
    };

    match result {
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "STORE_NOT_FOUND",
                format!("Store not found: {}", store_id),
            )),
        )
            .into_response()),
        Some(Some(org)) if org == my_org_id => Ok(()),
        Some(_) => Err(forbidden(
            "CROSS_ORG_STORE_DENIED",
            "Store belongs to a different organization",
        )),
    }
}

/// Drops every cached `(store_id → org_id)` entry. Intended for tests; v1.3
/// may also call this from store-mutation endpoints to invalidate eagerly.
pub fn clear_store_org_cache() {
    if let Ok(mut g) = store_org_cache().inner.write() {
        g.clear();
    }
}

/// Process-wide cache for `verify_store_in_org`. Stores `Option<Option<Uuid>>`
/// to mirror the DB query shape exactly: outer `None` = no row at all (404
/// territory), `Some(None)` = row exists with NULL org (transitional state
/// during the v1.0 backfill — should be empty post-v1.3), `Some(Some(uuid))`
/// = row exists with a real org. Caching all three states avoids leaking a
/// retry-storm pattern when a client hammers a bad id.
struct StoreOrgCache {
    inner: RwLock<HashMap<Uuid, CacheEntry>>,
}

#[derive(Clone, Copy)]
struct CacheEntry {
    resolved: Option<Option<Uuid>>,
    expires_at: Instant,
}

impl StoreOrgCache {
    fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    fn get(&self, id: Uuid) -> Option<Option<Option<Uuid>>> {
        let g = self.inner.read().ok()?;
        let entry = g.get(&id)?;
        if entry.expires_at > Instant::now() {
            Some(entry.resolved)
        } else {
            None
        }
    }

    fn put(&self, id: Uuid, resolved: Option<Option<Uuid>>) {
        if let Ok(mut g) = self.inner.write() {
            g.insert(
                id,
                CacheEntry {
                    resolved,
                    expires_at: Instant::now() + STORE_ORG_CACHE_TTL,
                },
            );
        }
    }
}

fn store_org_cache() -> &'static StoreOrgCache {
    static CACHE: OnceLock<StoreOrgCache> = OnceLock::new();
    CACHE.get_or_init(StoreOrgCache::new)
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{NoContext, Timestamp};

    fn id() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    #[test]
    fn cache_round_trips_all_three_resolved_states() {
        let cache = StoreOrgCache::new();
        let id_a = id();
        let id_b = id();
        let id_c = id();
        let org = id();

        cache.put(id_a, Some(Some(org)));
        cache.put(id_b, Some(None));
        cache.put(id_c, None);

        assert_eq!(cache.get(id_a), Some(Some(Some(org))));
        assert_eq!(cache.get(id_b), Some(Some(None)));
        assert_eq!(cache.get(id_c), Some(None));
    }

    #[test]
    fn cache_miss_for_unknown_id() {
        let cache = StoreOrgCache::new();
        assert_eq!(cache.get(id()), None);
    }

    #[test]
    fn cache_entry_expires_after_ttl() {
        let cache = StoreOrgCache::new();
        let key = id();
        if let Ok(mut g) = cache.inner.write() {
            g.insert(
                key,
                CacheEntry {
                    resolved: Some(Some(id())),
                    expires_at: Instant::now() - Duration::from_secs(1),
                },
            );
        }
        assert_eq!(
            cache.get(key),
            None,
            "expired entry must be treated as miss"
        );
    }
}
