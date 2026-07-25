// Audit log read handler
//
//   GET /backoffice/audit — platform:audit.read
//
// Exposes the append-only `backoffice_audit_log` over HTTP. Until this existed
// the platform wrote a complete audit trail that could only be inspected with a
// psql session — every mutation was recorded and none of it was reachable.
//
// Read-only, so no audit event of its own is emitted (same policy as the
// cross-org analytics reads). If reads themselves ever need to be audited, this
// is the single place that changes.

use axum::{
    Extension, Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use audit_infra::{AuditLogFilters, BackofficeAuditLogRecord};

use crate::error::AppError;
use crate::middleware::auth::BackofficeUserContext;
use crate::middleware::permission::require_backoffice_permission;
use crate::state::BackofficeAppState;

/// Rows returned when `page_size` is omitted.
const DEFAULT_PAGE_SIZE: u32 = 50;

/// Upper bound on `page_size`. The table grows without bound, so an
/// unclamped value is a trivial way to ask for the whole log in one request.
const MAX_PAGE_SIZE: u32 = 200;

/// `GET /backoffice/audit` query string.
///
/// Every field is optional. An unparseable `actor_id` / `target_org_id` /
/// `page` is rejected by axum's own deserialization with 400 before the
/// handler runs.
#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    /// Only entries produced by this backoffice operator.
    pub actor_id: Option<Uuid>,
    /// Only entries affecting this organization.
    pub target_org_id: Option<Uuid>,
    /// Exact-match action, e.g. `org.suspend`.
    pub action: Option<String>,
    /// 1-based page number. Values below 1 are treated as 1.
    pub page: Option<u32>,
    /// Rows per page, clamped to [`MAX_PAGE_SIZE`].
    pub page_size: Option<u32>,
}

/// Normalizes `page` to a 1-based value.
fn resolve_page(requested: Option<u32>) -> u32 {
    requested.unwrap_or(1).max(1)
}

/// Applies the default and the hard ceiling to `page_size`.
///
/// A requested `0` would return an empty page forever, which reads as "the log
/// is empty" rather than "you asked for nothing" — so it is floored to 1.
fn resolve_page_size(requested: Option<u32>) -> u32 {
    requested
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE)
}

/// One audit row as returned by the API.
///
/// A dedicated DTO rather than serializing the domain record: the wire shape is
/// an API concern, and the domain type must stay free to change without
/// silently altering the contract.
#[derive(Debug, Serialize)]
pub struct AuditLogEntryResponse {
    pub id: Uuid,
    pub actor_type: String,
    pub actor_id: Uuid,
    pub action: String,
    pub target_org_id: Option<Uuid>,
    pub reason: String,
    pub ip: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

impl From<BackofficeAuditLogRecord> for AuditLogEntryResponse {
    fn from(record: BackofficeAuditLogRecord) -> Self {
        Self {
            id: record.id,
            actor_type: record.actor_type,
            actor_id: record.actor_id,
            action: record.action,
            target_org_id: record.target_org_id,
            reason: record.reason,
            ip: record.ip,
            occurred_at: record.occurred_at,
        }
    }
}

/// A page of audit entries, newest first.
///
/// Deliberately carries no total count: `COUNT(*)` over an append-only table
/// that grows forever gets slower with every write, and it would run on every
/// request. Clients page until they receive fewer than `page_size` rows.
#[derive(Debug, Serialize)]
pub struct AuditLogPage {
    pub items: Vec<AuditLogEntryResponse>,
    pub page: u32,
    pub page_size: u32,
}

/// GET /backoffice/audit — paginated, newest first, `platform:audit.read`.
pub async fn list_audit_log_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Query(query): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:audit.read")?;

    let page = resolve_page(query.page);
    let page_size = resolve_page_size(query.page_size);

    let filters = AuditLogFilters {
        actor_id: query.actor_id,
        target_org_id: query.target_org_id,
        action: query.action,
    };

    let records = state
        .audit_log_repo()
        .find_paginated(filters, page, page_size)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(AuditLogPage {
        items: records.into_iter().map(Into::into).collect(),
        page,
        page_size,
    }))
}

// =============================================================================
// Tests — pagination normalization
//
// The query itself needs a live DB and is covered by the DB-backed repository
// tests; what is unit-testable here is the clamping that protects it.
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_defaults_to_one() {
        assert_eq!(resolve_page(None), 1);
    }

    /// Page 0 is not a valid 1-based page — it would compute a negative offset.
    #[test]
    fn page_zero_is_floored_to_one() {
        assert_eq!(resolve_page(Some(0)), 1);
    }

    #[test]
    fn page_is_otherwise_passed_through() {
        assert_eq!(resolve_page(Some(7)), 7);
    }

    #[test]
    fn page_size_defaults_when_omitted() {
        assert_eq!(resolve_page_size(None), DEFAULT_PAGE_SIZE);
    }

    /// The ceiling is what stops a caller pulling the entire log in one request.
    #[test]
    fn page_size_is_capped_at_the_maximum() {
        assert_eq!(resolve_page_size(Some(10_000)), MAX_PAGE_SIZE);
        assert_eq!(resolve_page_size(Some(u32::MAX)), MAX_PAGE_SIZE);
    }

    /// Zero rows per page would look like an empty log rather than a bad request.
    #[test]
    fn page_size_zero_is_floored_to_one() {
        assert_eq!(resolve_page_size(Some(0)), 1);
    }

    #[test]
    fn page_size_within_bounds_is_passed_through() {
        assert_eq!(resolve_page_size(Some(25)), 25);
        assert_eq!(resolve_page_size(Some(MAX_PAGE_SIZE)), MAX_PAGE_SIZE);
    }

    /// The extreme pair must not overflow when the repository computes
    /// `(page - 1) * page_size` for the SQL OFFSET.
    #[test]
    fn max_page_and_page_size_do_not_overflow_the_offset() {
        let page = resolve_page(Some(u32::MAX));
        let page_size = resolve_page_size(Some(u32::MAX));
        let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
        assert!(offset > 0, "offset must stay positive and representable");
    }

    #[test]
    fn record_maps_to_response_preserving_every_field() {
        use uuid::{NoContext, Timestamp};
        let id = Uuid::new_v7(Timestamp::now(NoContext));
        let actor_id = Uuid::new_v7(Timestamp::now(NoContext));
        let org_id = Uuid::new_v7(Timestamp::now(NoContext));
        let occurred_at = chrono::Utc::now();

        let response: AuditLogEntryResponse = BackofficeAuditLogRecord {
            id,
            actor_type: "backoffice_user".to_string(),
            actor_id,
            action: "org.suspend".to_string(),
            target_org_id: Some(org_id),
            reason: "fraud detected".to_string(),
            ip: "203.0.113.42".to_string(),
            occurred_at,
        }
        .into();

        assert_eq!(response.id, id);
        assert_eq!(response.actor_type, "backoffice_user");
        assert_eq!(response.actor_id, actor_id);
        assert_eq!(response.action, "org.suspend");
        assert_eq!(response.target_org_id, Some(org_id));
        assert_eq!(response.reason, "fraud detected");
        assert_eq!(response.ip, "203.0.113.42");
        assert_eq!(response.occurred_at, occurred_at);
    }
}
