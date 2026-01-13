// PostgreSQL RoleRepository implementation
//
// Requirements: 2.1, 2.3, 2.4, 2.5

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::{Permission, Role};
use crate::domain::repositories::RoleRepository;
use crate::domain::value_objects::{PermissionCode, PermissionId, RoleId};
use crate::error::IdentityError;

/// PostgreSQL implementation of RoleRepository
pub struct PgRoleRepository {
    pool: PgPool,
}

impl PgRoleRepository {
    /// Creates a new PgRoleRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepository for PgRoleRepository {
    async fn save(&self, role: &Role) -> Result<(), IdentityError> {
        sqlx::query(
            r#"
            INSERT INTO roles (id, name, description, is_system_protected, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(role.id().as_uuid())
        .bind(role.name())
        .bind(role.description())
        .bind(role.is_system_protected())
        .bind(role.created_at())
        .bind(role.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                IdentityError::DuplicateRole(role.name().to_string())
            }
            _ => IdentityError::Database(e),
        })?;

        Ok(())
    }

    async fn find_by_id(&self, id: RoleId) -> Result<Option<Role>, IdentityError> {
        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, description, is_system_protected, created_at, updated_at
            FROM roles
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }


    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, IdentityError> {
        let row = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, description, is_system_protected, created_at, updated_at
            FROM roles
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_all(&self) -> Result<Vec<Role>, IdentityError> {
        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT id, name, description, is_system_protected, created_at, updated_at
            FROM roles
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn delete(&self, id: RoleId) -> Result<(), IdentityError> {
        // First check if the role exists and is not system-protected
        let role = self.find_by_id(id).await?;
        match role {
            None => return Err(IdentityError::RoleNotFound(id.into_uuid())),
            Some(r) if r.is_system_protected() => {
                return Err(IdentityError::ProtectedRoleCannotBeDeleted)
            }
            _ => {}
        }

        // Delete the role (cascades will handle role_permissions and user_store_roles)
        sqlx::query(
            r#"
            DELETE FROM roles
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, role: &Role) -> Result<(), IdentityError> {
        let result = sqlx::query(
            r#"
            UPDATE roles
            SET name = $2, description = $3, is_system_protected = $4, updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(role.id().as_uuid())
        .bind(role.name())
        .bind(role.description())
        .bind(role.is_system_protected())
        .bind(role.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                IdentityError::DuplicateRole(role.name().to_string())
            }
            _ => IdentityError::Database(e),
        })?;

        if result.rows_affected() == 0 {
            return Err(IdentityError::RoleNotFound(role.id().into_uuid()));
        }

        Ok(())
    }


    async fn add_permission(
        &self,
        role_id: RoleId,
        permission_id: PermissionId,
    ) -> Result<(), IdentityError> {
        // Verify role exists
        let role_exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM roles WHERE id = $1)"#,
        )
        .bind(role_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        if !role_exists {
            return Err(IdentityError::RoleNotFound(role_id.into_uuid()));
        }

        // Verify permission exists
        let permission_exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM permissions WHERE id = $1)"#,
        )
        .bind(permission_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        if !permission_exists {
            return Err(IdentityError::PermissionNotFound(permission_id.into_uuid()));
        }

        // Insert the relationship (ignore if already exists)
        sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission_id, created_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (role_id, permission_id) DO NOTHING
            "#,
        )
        .bind(role_id.as_uuid())
        .bind(permission_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_permission(
        &self,
        role_id: RoleId,
        permission_id: PermissionId,
    ) -> Result<(), IdentityError> {
        // Verify role exists
        let role_exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM roles WHERE id = $1)"#,
        )
        .bind(role_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        if !role_exists {
            return Err(IdentityError::RoleNotFound(role_id.into_uuid()));
        }

        sqlx::query(
            r#"
            DELETE FROM role_permissions
            WHERE role_id = $1 AND permission_id = $2
            "#,
        )
        .bind(role_id.as_uuid())
        .bind(permission_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_permissions(&self, role_id: RoleId) -> Result<Vec<Permission>, IdentityError> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT p.id, p.code, p.description, p.created_at
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            ORDER BY p.code
            "#,
        )
        .bind(role_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn remove_permission_from_all_roles(
        &self,
        permission_id: PermissionId,
    ) -> Result<(), IdentityError> {
        sqlx::query(
            r#"
            DELETE FROM role_permissions
            WHERE permission_id = $1
            "#,
        )
        .bind(permission_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Internal row type for mapping database results
#[derive(sqlx::FromRow)]
struct RoleRow {
    id: uuid::Uuid,
    name: String,
    description: Option<String>,
    is_system_protected: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<RoleRow> for Role {
    fn from(row: RoleRow) -> Self {
        Role::new(
            RoleId::from_uuid(row.id),
            row.name,
            row.description,
            row.is_system_protected,
            row.created_at,
            row.updated_at,
        )
    }
}

/// Internal row type for permission mapping
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
