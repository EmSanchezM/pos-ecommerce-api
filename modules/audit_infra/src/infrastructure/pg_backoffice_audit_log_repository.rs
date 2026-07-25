//! PostgreSQL implementation of `BackofficeAuditLogRepository`.
//!
//! Uses only `INSERT INTO backoffice_audit_log` — no UPDATE or DELETE.
//! This enforces FR-AUD-4 / NFR-SEC-2 at the infrastructure layer.

use async_trait::async_trait;
use uuid::{NoContext, Timestamp, Uuid};

use sqlx::PgPool;

use crate::AuditInfraError;
use crate::domain::repositories::{
    AuditLogFilters, BackofficeAuditLogEntry, BackofficeAuditLogRecord,
    BackofficeAuditLogRepository,
};

/// PostgreSQL implementation — append-only.
#[derive(Clone)]
pub struct PgBackofficeAuditLogRepository {
    pool: PgPool,
}

impl PgBackofficeAuditLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BackofficeAuditLogRepository for PgBackofficeAuditLogRepository {
    /// Inserts a single immutable row into `backoffice_audit_log`.
    ///
    /// UUID v7 is generated here so every row has a monotonically increasing
    /// primary key (important for efficient range-scan pagination later).
    async fn append(&self, entry: BackofficeAuditLogEntry) -> Result<(), AuditInfraError> {
        let id = Uuid::new_v7(Timestamp::now(NoContext));

        sqlx::query(
            r#"
            INSERT INTO backoffice_audit_log
                (id, actor_id, actor_type, action, target_org_id, reason, ip)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(entry.actor_id)
        .bind(&entry.actor_type)
        .bind(&entry.action)
        .bind(entry.target_org_id)
        .bind(&entry.reason)
        .bind(&entry.ip)
        .execute(&self.pool)
        .await
        .map_err(AuditInfraError::Database)?;

        Ok(())
    }

    /// Returns paginated rows ordered by `occurred_at DESC`.
    ///
    /// Optional filters: `actor_id`, `target_org_id`, `action`.
    /// Pagination: 1-based page number, `page_size` rows per page.
    async fn find_paginated(
        &self,
        filters: AuditLogFilters,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<BackofficeAuditLogRecord>, AuditInfraError> {
        // Widen to i64 BEFORE multiplying: `page * page_size` in u32 overflows
        // (a debug-build panic) for large values, and both come from query
        // params on a public endpoint.
        let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
        let limit = i64::from(page_size);

        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT id, actor_id, actor_type, action, target_org_id, reason, ip, occurred_at
            FROM backoffice_audit_log
            WHERE
                ($1::uuid IS NULL OR actor_id = $1)
            AND ($2::uuid IS NULL OR target_org_id = $2)
            AND ($3::text IS NULL OR action = $3)
            ORDER BY occurred_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(filters.actor_id)
        .bind(filters.target_org_id)
        .bind(filters.action)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(AuditInfraError::Database)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

// =============================================================================
// Internal row struct for sqlx::query_as mapping
// =============================================================================

#[derive(sqlx::FromRow)]
struct AuditLogRow {
    id: Uuid,
    actor_id: Uuid,
    actor_type: String,
    action: String,
    target_org_id: Option<Uuid>,
    reason: String,
    ip: String,
    occurred_at: chrono::DateTime<chrono::Utc>,
}

impl From<AuditLogRow> for BackofficeAuditLogRecord {
    fn from(row: AuditLogRow) -> Self {
        Self {
            id: row.id,
            actor_type: row.actor_type,
            actor_id: row.actor_id,
            action: row.action,
            target_org_id: row.target_org_id,
            reason: row.reason,
            ip: row.ip,
            occurred_at: row.occurred_at,
        }
    }
}

// =============================================================================
// DB-backed tests
//
// The read query is the whole point of this repository and it cannot be
// exercised by a mock: column names, the NULL-guarded filters, the ordering and
// the OFFSET arithmetic are only real against Postgres. These require a live
// database (`docker compose -f compose.dev.yml up -d db`); without one they
// fail with PoolTimedOut rather than passing vacuously.
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn entry(action: &str, actor_id: Uuid, target_org_id: Option<Uuid>) -> BackofficeAuditLogEntry {
        BackofficeAuditLogEntry {
            actor_type: "backoffice_user".to_string(),
            actor_id,
            action: action.to_string(),
            target_org_id,
            reason: format!("reason for {action}"),
            ip: "203.0.113.42".to_string(),
        }
    }

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    /// The columns Postgres assigns on INSERT must come back on read — an audit
    /// row without `occurred_at` cannot answer "when did this happen".
    #[sqlx::test(migrations = "../../migrations")]
    async fn read_returns_db_assigned_id_and_timestamp(pool: sqlx::PgPool) {
        let repo = PgBackofficeAuditLogRepository::new(pool);
        let actor = new_uuid();

        repo.append(entry("org.suspend", actor, None))
            .await
            .expect("append should succeed");

        let rows = repo
            .find_paginated(AuditLogFilters::default(), 1, 10)
            .await
            .expect("read should succeed");

        assert_eq!(rows.len(), 1);
        assert!(!rows[0].id.is_nil(), "id must be populated from the DB");
        assert_eq!(rows[0].actor_id, actor);
        assert_eq!(rows[0].action, "org.suspend");
        assert_eq!(rows[0].ip, "203.0.113.42");
        assert!(
            rows[0].occurred_at.timestamp() > 0,
            "occurred_at must be populated from the DB default"
        );
    }

    /// Newest first — an audit reader starts from the most recent action.
    #[sqlx::test(migrations = "../../migrations")]
    async fn rows_come_back_newest_first(pool: sqlx::PgPool) {
        let repo = PgBackofficeAuditLogRepository::new(pool);
        let actor = new_uuid();

        repo.append(entry("first.action", actor, None))
            .await
            .unwrap();
        repo.append(entry("second.action", actor, None))
            .await
            .unwrap();

        let rows = repo
            .find_paginated(AuditLogFilters::default(), 1, 10)
            .await
            .unwrap();

        assert_eq!(rows.len(), 2);
        assert!(
            rows[0].occurred_at >= rows[1].occurred_at,
            "rows must be ordered by occurred_at DESC"
        );
    }

    /// An omitted filter must not filter — the NULL guards in the WHERE clause
    /// are easy to get backwards.
    #[sqlx::test(migrations = "../../migrations")]
    async fn default_filters_return_every_row(pool: sqlx::PgPool) {
        let repo = PgBackofficeAuditLogRepository::new(pool);

        repo.append(entry("a.one", new_uuid(), None)).await.unwrap();
        repo.append(entry("b.two", new_uuid(), Some(new_uuid())))
            .await
            .unwrap();

        let rows = repo
            .find_paginated(AuditLogFilters::default(), 1, 10)
            .await
            .unwrap();

        assert_eq!(rows.len(), 2);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn filters_by_actor_id(pool: sqlx::PgPool) {
        let repo = PgBackofficeAuditLogRepository::new(pool);
        let wanted = new_uuid();

        repo.append(entry("org.suspend", wanted, None))
            .await
            .unwrap();
        repo.append(entry("org.suspend", new_uuid(), None))
            .await
            .unwrap();

        let rows = repo
            .find_paginated(
                AuditLogFilters {
                    actor_id: Some(wanted),
                    ..Default::default()
                },
                1,
                10,
            )
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].actor_id, wanted);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn filters_by_target_org_id(pool: sqlx::PgPool) {
        let repo = PgBackofficeAuditLogRepository::new(pool);
        let org = new_uuid();

        repo.append(entry("org.suspend", new_uuid(), Some(org)))
            .await
            .unwrap();
        repo.append(entry("org.suspend", new_uuid(), Some(new_uuid())))
            .await
            .unwrap();
        // A row with no target org must not match an org filter.
        repo.append(entry("plan.create", new_uuid(), None))
            .await
            .unwrap();

        let rows = repo
            .find_paginated(
                AuditLogFilters {
                    target_org_id: Some(org),
                    ..Default::default()
                },
                1,
                10,
            )
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target_org_id, Some(org));
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn filters_by_action(pool: sqlx::PgPool) {
        let repo = PgBackofficeAuditLogRepository::new(pool);

        repo.append(entry("user.impersonate", new_uuid(), None))
            .await
            .unwrap();
        repo.append(entry("org.suspend", new_uuid(), None))
            .await
            .unwrap();

        let rows = repo
            .find_paginated(
                AuditLogFilters {
                    action: Some("user.impersonate".to_string()),
                    ..Default::default()
                },
                1,
                10,
            )
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].action, "user.impersonate");
    }

    /// Page 2 must not repeat page 1 — an off-by-one in the OFFSET would show
    /// the same row twice and silently hide one.
    #[sqlx::test(migrations = "../../migrations")]
    async fn pagination_does_not_repeat_or_skip_rows(pool: sqlx::PgPool) {
        let repo = PgBackofficeAuditLogRepository::new(pool);
        let actor = new_uuid();

        for i in 0..3 {
            repo.append(entry(&format!("action.{i}"), actor, None))
                .await
                .unwrap();
        }

        let page1 = repo
            .find_paginated(AuditLogFilters::default(), 1, 2)
            .await
            .unwrap();
        let page2 = repo
            .find_paginated(AuditLogFilters::default(), 2, 2)
            .await
            .unwrap();

        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 1);

        let page1_ids: Vec<Uuid> = page1.iter().map(|r| r.id).collect();
        assert!(
            !page1_ids.contains(&page2[0].id),
            "page 2 must not repeat a row from page 1"
        );
    }

    /// Page 0 is not a valid 1-based page; `saturating_sub` must keep the
    /// OFFSET at 0 rather than wrapping to a huge value.
    #[sqlx::test(migrations = "../../migrations")]
    async fn page_zero_behaves_like_page_one(pool: sqlx::PgPool) {
        let repo = PgBackofficeAuditLogRepository::new(pool);
        repo.append(entry("org.suspend", new_uuid(), None))
            .await
            .unwrap();

        let rows = repo
            .find_paginated(AuditLogFilters::default(), 0, 10)
            .await
            .expect("page 0 must not produce an invalid OFFSET");

        assert_eq!(rows.len(), 1);
    }

    /// Reading an empty log is a normal outcome, not an error.
    #[sqlx::test(migrations = "../../migrations")]
    async fn empty_log_returns_an_empty_page(pool: sqlx::PgPool) {
        let repo = PgBackofficeAuditLogRepository::new(pool);

        let rows = repo
            .find_paginated(AuditLogFilters::default(), 1, 10)
            .await
            .unwrap();

        assert!(rows.is_empty());
    }
}
