//! PostgreSQL implementation of `BackofficeAuditLogRepository`.
//!
//! Uses only `INSERT INTO backoffice_audit_log` — no UPDATE or DELETE.
//! This enforces FR-AUD-4 / NFR-SEC-2 at the infrastructure layer.

use async_trait::async_trait;
use uuid::{NoContext, Timestamp, Uuid};

use sqlx::PgPool;

use crate::AuditInfraError;
use crate::domain::repositories::{
    AuditLogFilters, BackofficeAuditLogEntry, BackofficeAuditLogRepository,
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
    ) -> Result<Vec<BackofficeAuditLogEntry>, AuditInfraError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;

        let rows = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT actor_id, actor_type, action, target_org_id, reason, ip
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
    actor_id: Uuid,
    actor_type: String,
    action: String,
    target_org_id: Option<Uuid>,
    reason: String,
    ip: String,
}

impl From<AuditLogRow> for BackofficeAuditLogEntry {
    fn from(row: AuditLogRow) -> Self {
        Self {
            actor_type: row.actor_type,
            actor_id: row.actor_id,
            action: row.action,
            target_org_id: row.target_org_id,
            reason: row.reason,
            ip: row.ip,
        }
    }
}
