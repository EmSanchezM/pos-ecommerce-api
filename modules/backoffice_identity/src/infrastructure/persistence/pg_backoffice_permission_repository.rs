use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::BackofficePermission;
use crate::domain::repositories::BackofficePermissionRepository;
use crate::domain::value_objects::{BackofficePermissionId, PlatformPermissionCode};
use crate::error::BackofficeIdentityError;

pub struct PgBackofficePermissionRepository {
    pool: PgPool,
}

impl PgBackofficePermissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PermissionRow {
    id: uuid::Uuid,
    code: String,
    description: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<PermissionRow> for BackofficePermission {
    type Error = BackofficeIdentityError;

    fn try_from(r: PermissionRow) -> Result<Self, Self::Error> {
        let code = PlatformPermissionCode::new(&r.code)
            .map_err(|_| BackofficeIdentityError::InvalidPermissionCodeFormat)?;
        Ok(BackofficePermission::new(
            BackofficePermissionId::from_uuid(r.id),
            code,
            r.description,
            r.created_at,
        ))
    }
}

#[async_trait]
impl BackofficePermissionRepository for PgBackofficePermissionRepository {
    async fn save(&self, permission: &BackofficePermission) -> Result<(), BackofficeIdentityError> {
        sqlx::query(
            r#"
            INSERT INTO backoffice_permissions (id, code, description, created_at)
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
                BackofficeIdentityError::DuplicatePermission(permission.code().as_str().to_string())
            }
            _ => BackofficeIdentityError::Database(e),
        })?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: BackofficePermissionId,
    ) -> Result<Option<BackofficePermission>, BackofficeIdentityError> {
        let row: Option<PermissionRow> = sqlx::query_as(
            r#"
            SELECT id, code, description, created_at
            FROM backoffice_permissions
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(BackofficePermission::try_from).transpose()
    }

    async fn find_by_code(
        &self,
        code: &PlatformPermissionCode,
    ) -> Result<Option<BackofficePermission>, BackofficeIdentityError> {
        let row: Option<PermissionRow> = sqlx::query_as(
            r#"
            SELECT id, code, description, created_at
            FROM backoffice_permissions
            WHERE code = $1
            "#,
        )
        .bind(code.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(BackofficePermission::try_from).transpose()
    }

    async fn list(&self) -> Result<Vec<BackofficePermission>, BackofficeIdentityError> {
        let rows: Vec<PermissionRow> = sqlx::query_as(
            r#"
            SELECT id, code, description, created_at
            FROM backoffice_permissions
            ORDER BY code ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(BackofficePermission::try_from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{NoContext, Timestamp, Uuid};

    fn make_code(s: &str) -> PlatformPermissionCode {
        PlatformPermissionCode::new(s).unwrap()
    }

    fn make_permission(code: &str) -> BackofficePermission {
        BackofficePermission::new(
            BackofficePermissionId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext))),
            make_code(code),
            Some(format!("Description for {}", code)),
            chrono::Utc::now(),
        )
    }

    #[test]
    fn test_pg_permission_repository_struct_exists() {
        let _type_check: fn(PgPool) -> PgBackofficePermissionRepository =
            PgBackofficePermissionRepository::new;
        fn assert_impl<T: BackofficePermissionRepository>() {}
        assert_impl::<PgBackofficePermissionRepository>();
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn save_then_find_by_id_returns_permission(pool: PgPool) {
        let repo = PgBackofficePermissionRepository::new(pool);
        let perm = make_permission("platform:test.fixture_id");
        let perm_id = *perm.id();

        repo.save(&perm).await.expect("save should succeed");

        let found = repo
            .find_by_id(perm_id)
            .await
            .expect("find_by_id should not error")
            .expect("permission should exist");

        assert_eq!(found.id(), &perm_id);
        assert_eq!(found.code().as_str(), "platform:test.fixture_id");
        assert_eq!(
            found.description(),
            Some("Description for platform:test.fixture_id")
        );
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn save_then_find_by_code_returns_permission(pool: PgPool) {
        let repo = PgBackofficePermissionRepository::new(pool);
        let perm = make_permission("platform:test.fixture_code");
        let perm_id = *perm.id();

        repo.save(&perm).await.expect("save should succeed");

        let code = make_code("platform:test.fixture_code");
        let found = repo
            .find_by_code(&code)
            .await
            .expect("find_by_code should not error")
            .expect("permission should exist");

        assert_eq!(found.id(), &perm_id);
        assert_eq!(found.code().as_str(), "platform:test.fixture_code");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn find_by_id_missing_returns_none(pool: PgPool) {
        let repo = PgBackofficePermissionRepository::new(pool);
        let missing_id = BackofficePermissionId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));

        let result = repo
            .find_by_id(missing_id)
            .await
            .expect("find_by_id should not error on missing id");

        assert!(result.is_none());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn find_by_code_missing_returns_none(pool: PgPool) {
        let repo = PgBackofficePermissionRepository::new(pool);
        // sqlx::test runs migrations on each test DB, so seeded codes like
        // platform:org.create will exist. Use a code that is NOT seeded at all.
        let nonexistent = make_code("platform:missing.action");
        let result = repo
            .find_by_code(&nonexistent)
            .await
            .expect("find_by_code should not error on missing code");

        assert!(result.is_none());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn save_duplicate_code_returns_duplicate_permission_error(pool: PgPool) {
        let repo = PgBackofficePermissionRepository::new(pool);
        let perm1 = make_permission("platform:test.duplicate");
        let perm2 = BackofficePermission::new(
            BackofficePermissionId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext))),
            make_code("platform:test.duplicate"),
            None,
            chrono::Utc::now(),
        );

        repo.save(&perm1).await.expect("first save should succeed");

        let err = repo
            .save(&perm2)
            .await
            .expect_err("second save with duplicate code should fail");

        assert!(
            matches!(err, BackofficeIdentityError::DuplicatePermission(_)),
            "expected DuplicatePermission, got: {:?}",
            err
        );
    }
}
