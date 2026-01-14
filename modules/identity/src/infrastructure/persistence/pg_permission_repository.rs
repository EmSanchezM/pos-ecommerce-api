// PostgreSQL PermissionRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::Permission;
use crate::domain::repositories::PermissionRepository;
use crate::domain::value_objects::{PermissionCode, PermissionId};
use crate::error::IdentityError;

/// PostgreSQL implementation of PermissionRepository
pub struct PgPermissionRepository {
    pool: PgPool,
}

impl PgPermissionRepository {
    /// Creates a new PgPermissionRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PermissionRepository for PgPermissionRepository {
    async fn save(&self, permission: &Permission) -> Result<(), IdentityError> {
        sqlx::query(
            r#"
            INSERT INTO permissions (id, code, description, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(permission.id().as_uuid())
        .bind(permission.code().as_str())
        .bind(permission.description())
        .bind(permission.created_at())
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                IdentityError::DuplicatePermission(permission.code().as_str().to_string())
            }
            _ => IdentityError::Database(e),
        })?;

        Ok(())
    }

    async fn find_by_id(&self, id: PermissionId) -> Result<Option<Permission>, IdentityError> {
        let row = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT id, code, description, created_at
            FROM permissions
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }


    async fn find_by_code(&self, code: &PermissionCode) -> Result<Option<Permission>, IdentityError> {
        let row = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT id, code, description, created_at
            FROM permissions
            WHERE code = $1
            "#,
        )
        .bind(code.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_all(&self) -> Result<Vec<Permission>, IdentityError> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT id, code, description, created_at
            FROM permissions
            ORDER BY code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_by_module(&self, module: &str) -> Result<Vec<Permission>, IdentityError> {
        let pattern = format!("{}:%", module);
        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT id, code, description, created_at
            FROM permissions
            WHERE code LIKE $1
            ORDER BY code
            "#,
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn delete(&self, id: PermissionId) -> Result<(), IdentityError> {
        let result = sqlx::query(
            r#"
            DELETE FROM permissions
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(IdentityError::PermissionNotFound(id.into_uuid()));
        }

        Ok(())
    }

    async fn exists(&self, code: &PermissionCode) -> Result<bool, IdentityError> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(SELECT 1 FROM permissions WHERE code = $1)
            "#,
        )
        .bind(code.as_str())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }
}

/// Internal row type for mapping database results
#[derive(sqlx::FromRow)]
struct PermissionRow {
    id: uuid::Uuid,
    code: String,
    description: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<PermissionRow> for Permission {
    type Error = IdentityError;

    fn try_from(row: PermissionRow) -> Result<Self, Self::Error> {
        let code = PermissionCode::new(&row.code)?;
        Ok(Permission::new(
            PermissionId::from_uuid(row.id),
            code,
            row.description,
            row.created_at,
        ))
    }
}
