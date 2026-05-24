use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::BackofficeRole;
use crate::domain::repositories::BackofficeRoleRepository;
use crate::domain::value_objects::BackofficeRoleId;
use crate::error::BackofficeIdentityError;

pub struct PgBackofficeRoleRepository {
    pool: PgPool,
}

impl PgBackofficeRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RoleRow {
    id: uuid::Uuid,
    name: String,
    description: Option<String>,
    is_system_protected: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<RoleRow> for BackofficeRole {
    fn from(r: RoleRow) -> Self {
        BackofficeRole::new(
            BackofficeRoleId::from_uuid(r.id),
            r.name,
            r.description,
            r.is_system_protected,
            r.created_at,
        )
    }
}

#[async_trait]
impl BackofficeRoleRepository for PgBackofficeRoleRepository {
    async fn save(&self, role: &BackofficeRole) -> Result<(), BackofficeIdentityError> {
        sqlx::query(
            r#"
            INSERT INTO backoffice_roles (id, name, description, is_system_protected, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(role.id().as_uuid())
        .bind(role.name())
        .bind(role.description())
        .bind(role.is_system_protected())
        .bind(role.created_at())
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                BackofficeIdentityError::DuplicateRole(role.name().to_string())
            }
            _ => BackofficeIdentityError::Database(e),
        })?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: BackofficeRoleId,
    ) -> Result<Option<BackofficeRole>, BackofficeIdentityError> {
        let row: Option<RoleRow> = sqlx::query_as(
            r#"
            SELECT id, name, description, is_system_protected, created_at
            FROM backoffice_roles
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(BackofficeRole::from))
    }

    async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<BackofficeRole>, BackofficeIdentityError> {
        let row: Option<RoleRow> = sqlx::query_as(
            r#"
            SELECT id, name, description, is_system_protected, created_at
            FROM backoffice_roles
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(BackofficeRole::from))
    }

    async fn list(&self) -> Result<Vec<BackofficeRole>, BackofficeIdentityError> {
        let rows: Vec<RoleRow> = sqlx::query_as(
            r#"
            SELECT id, name, description, is_system_protected, created_at
            FROM backoffice_roles
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(BackofficeRole::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{NoContext, Timestamp, Uuid};

    fn make_role(name: &str, is_protected: bool) -> BackofficeRole {
        BackofficeRole::new(
            BackofficeRoleId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext))),
            name.to_string(),
            Some(format!("Description for {}", name)),
            is_protected,
            chrono::Utc::now(),
        )
    }

    #[test]
    fn test_pg_role_repository_struct_exists() {
        let _type_check: fn(PgPool) -> PgBackofficeRoleRepository = PgBackofficeRoleRepository::new;
        fn assert_impl<T: BackofficeRoleRepository>() {}
        assert_impl::<PgBackofficeRoleRepository>();
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn save_then_find_by_id_preserves_is_system_protected(pool: PgPool) {
        let repo = PgBackofficeRoleRepository::new(pool);
        let role = make_role("test_admin_role", true);
        let role_id = *role.id();

        repo.save(&role).await.expect("save should succeed");

        let found = repo
            .find_by_id(role_id)
            .await
            .expect("find_by_id should not error")
            .expect("role should exist");

        assert_eq!(found.id(), &role_id);
        assert_eq!(found.name(), "test_admin_role");
        assert_eq!(found.description(), Some("Description for test_admin_role"));
        assert!(found.is_system_protected(), "is_system_protected must survive round-trip");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn save_then_find_by_name_exact_match(pool: PgPool) {
        let repo = PgBackofficeRoleRepository::new(pool);
        let role = make_role("test_billing_role", false);
        let role_id = *role.id();

        repo.save(&role).await.expect("save should succeed");

        let found = repo
            .find_by_name("test_billing_role")
            .await
            .expect("find_by_name should not error")
            .expect("role should exist by name");

        assert_eq!(found.id(), &role_id);
        assert!(!found.is_system_protected());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn find_by_id_missing_returns_none(pool: PgPool) {
        let repo = PgBackofficeRoleRepository::new(pool);
        let missing_id = BackofficeRoleId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));

        let result = repo
            .find_by_id(missing_id)
            .await
            .expect("find_by_id should not error on missing id");

        assert!(result.is_none());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn find_by_name_missing_returns_none(pool: PgPool) {
        let repo = PgBackofficeRoleRepository::new(pool);

        let result = repo
            .find_by_name("nonexistent_role_xyz")
            .await
            .expect("find_by_name should not error on missing name");

        assert!(result.is_none());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn save_duplicate_name_returns_duplicate_role_error(pool: PgPool) {
        let repo = PgBackofficeRoleRepository::new(pool);
        let role1 = make_role("unique_test_role", false);
        let role2 = BackofficeRole::new(
            BackofficeRoleId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext))),
            "unique_test_role".to_string(),
            None,
            false,
            chrono::Utc::now(),
        );

        repo.save(&role1).await.expect("first save should succeed");

        let err = repo
            .save(&role2)
            .await
            .expect_err("second save with duplicate name should fail");

        assert!(
            matches!(err, BackofficeIdentityError::DuplicateRole(_)),
            "expected DuplicateRole, got: {:?}",
            err
        );
    }
}
