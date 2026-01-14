// PostgreSQL AuditRepository implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::{AuditAction, AuditEntry};
use crate::domain::repositories::AuditRepository;
use crate::domain::value_objects::UserId;
use crate::error::IdentityError;

/// PostgreSQL implementation of AuditRepository
pub struct PgAuditRepository {
    pool: PgPool,
}

impl PgAuditRepository {
    /// Creates a new PgAuditRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for PgAuditRepository {
    async fn save(&self, entry: &AuditEntry) -> Result<(), IdentityError> {
        sqlx::query(
            r#"
            INSERT INTO audit_log (id, entity_type, entity_id, action, old_value, new_value, actor_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(entry.id())
        .bind(entry.entity_type())
        .bind(entry.entity_id())
        .bind(entry.action().to_string())
        .bind(entry.old_value())
        .bind(entry.new_value())
        .bind(entry.actor_id().as_uuid())
        .bind(entry.created_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditEntry>, IdentityError> {
        let rows = sqlx::query_as::<_, AuditRow>(
            r#"
            SELECT id, entity_type, entity_id, action, old_value, new_value, actor_id, created_at
            FROM audit_log
            WHERE entity_type = $1 AND entity_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_date_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, IdentityError> {
        let rows = sqlx::query_as::<_, AuditRow>(
            r#"
            SELECT id, entity_type, entity_id, action, old_value, new_value, actor_id, created_at
            FROM audit_log
            WHERE created_at >= $1 AND created_at < $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

/// Internal row type for mapping audit database results
#[derive(sqlx::FromRow)]
struct AuditRow {
    id: Uuid,
    entity_type: String,
    entity_id: Uuid,
    action: String,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    actor_id: Uuid,
    created_at: DateTime<Utc>,
}

impl From<AuditRow> for AuditEntry {
    fn from(row: AuditRow) -> Self {
        let action = parse_audit_action(&row.action);
        AuditEntry::new(
            row.id,
            row.entity_type,
            row.entity_id,
            action,
            row.old_value,
            row.new_value,
            UserId::from_uuid(row.actor_id),
            row.created_at,
        )
    }
}

/// Parses an action string into an AuditAction enum
fn parse_audit_action(action: &str) -> AuditAction {
    match action {
        "created" => AuditAction::Created,
        "updated" => AuditAction::Updated,
        "deleted" => AuditAction::Deleted,
        "permission_added" => AuditAction::PermissionAdded,
        "permission_removed" => AuditAction::PermissionRemoved,
        "role_assigned" => AuditAction::RoleAssigned,
        "role_unassigned" => AuditAction::RoleUnassigned,
        "user_added_to_store" => AuditAction::UserAddedToStore,
        "user_removed_from_store" => AuditAction::UserRemovedFromStore,
        // Default to Created for unknown actions (shouldn't happen in practice)
        _ => AuditAction::Created,
    }
}
