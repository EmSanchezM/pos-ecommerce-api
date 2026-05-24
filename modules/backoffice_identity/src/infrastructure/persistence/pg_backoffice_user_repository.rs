use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::{BackofficePermission, BackofficeRole, BackofficeUser};
use crate::domain::repositories::BackofficeUserRepository;
use crate::domain::value_objects::{
    BackofficeEmail, BackofficePermissionId, BackofficeRoleId, BackofficeUserId,
    PlatformPermissionCode,
};
use crate::error::BackofficeIdentityError;

pub struct PgBackofficeUserRepository {
    pool: PgPool,
}

impl PgBackofficeUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    email: String,
    password_hash: String,
    mfa_secret: Option<String>,
    is_active: bool,
    last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<UserRow> for BackofficeUser {
    type Error = BackofficeIdentityError;

    fn try_from(r: UserRow) -> Result<Self, Self::Error> {
        let email = BackofficeEmail::new(&r.email)
            .map_err(|_| BackofficeIdentityError::InvalidEmailFormat)?;
        Ok(BackofficeUser::new(
            BackofficeUserId::from_uuid(r.id),
            email,
            r.password_hash,
            r.mfa_secret,
            r.is_active,
            r.last_login_at,
            r.created_at,
            r.updated_at,
        ))
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

#[derive(sqlx::FromRow)]
struct PermissionRow {
    id: uuid::Uuid,
    code: String,
    description: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
impl BackofficeUserRepository for PgBackofficeUserRepository {
    async fn save(&self, user: &BackofficeUser) -> Result<(), BackofficeIdentityError> {
        sqlx::query(
            r#"
            INSERT INTO backoffice_users
                (id, email, password_hash, mfa_secret, is_active, last_login_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(user.id().as_uuid())
        .bind(user.email().as_str())
        .bind(user.password_hash())
        .bind(user.mfa_secret())
        .bind(user.is_active())
        .bind(user.last_login_at())
        .bind(user.created_at())
        .bind(user.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                BackofficeIdentityError::DuplicateEmail(user.email().as_str().to_string())
            }
            _ => BackofficeIdentityError::Database(e),
        })?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: BackofficeUserId,
    ) -> Result<Option<BackofficeUser>, BackofficeIdentityError> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, email, password_hash, mfa_secret, is_active, last_login_at, created_at, updated_at
            FROM backoffice_users
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(BackofficeUser::try_from).transpose()
    }

    async fn find_by_email(
        &self,
        email: &BackofficeEmail,
    ) -> Result<Option<BackofficeUser>, BackofficeIdentityError> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, email, password_hash, mfa_secret, is_active, last_login_at, created_at, updated_at
            FROM backoffice_users
            WHERE email = $1
            "#,
        )
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(BackofficeUser::try_from).transpose()
    }

    async fn update(&self, user: &BackofficeUser) -> Result<(), BackofficeIdentityError> {
        sqlx::query(
            r#"
            UPDATE backoffice_users
            SET email = $1,
                password_hash = $2,
                mfa_secret = $3,
                is_active = $4,
                last_login_at = $5,
                updated_at = $6
            WHERE id = $7
            "#,
        )
        .bind(user.email().as_str())
        .bind(user.password_hash())
        .bind(user.mfa_secret())
        .bind(user.is_active())
        .bind(user.last_login_at())
        .bind(user.updated_at())
        .bind(user.id().as_uuid())
        .execute(&self.pool)
        .await
        .map_err(BackofficeIdentityError::Database)?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<BackofficeUser>, BackofficeIdentityError> {
        let rows: Vec<UserRow> = sqlx::query_as(
            r#"
            SELECT id, email, password_hash, mfa_secret, is_active, last_login_at, created_at, updated_at
            FROM backoffice_users
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(BackofficeUser::try_from).collect()
    }

    async fn list_roles_for_user(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<Vec<BackofficeRole>, BackofficeIdentityError> {
        let rows: Vec<RoleRow> = sqlx::query_as(
            r#"
            SELECT br.id, br.name, br.description, br.is_system_protected, br.created_at
            FROM backoffice_roles br
            INNER JOIN backoffice_user_roles bur ON bur.role_id = br.id
            WHERE bur.backoffice_user_id = $1
            ORDER BY br.name ASC
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                BackofficeRole::new(
                    BackofficeRoleId::from_uuid(r.id),
                    r.name,
                    r.description,
                    r.is_system_protected,
                    r.created_at,
                )
            })
            .collect())
    }

    async fn list_permissions_for_user(
        &self,
        user_id: BackofficeUserId,
    ) -> Result<Vec<BackofficePermission>, BackofficeIdentityError> {
        let rows: Vec<PermissionRow> = sqlx::query_as(
            r#"
            SELECT DISTINCT bp.id, bp.code, bp.description, bp.created_at
            FROM backoffice_permissions bp
            INNER JOIN backoffice_role_permissions brp ON brp.permission_id = bp.id
            INNER JOIN backoffice_user_roles bur ON bur.role_id = brp.role_id
            WHERE bur.backoffice_user_id = $1
            ORDER BY bp.code ASC
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| {
                let code = PlatformPermissionCode::new(&r.code)
                    .map_err(|_| BackofficeIdentityError::InvalidPermissionCodeFormat)?;
                Ok(BackofficePermission::new(
                    BackofficePermissionId::from_uuid(r.id),
                    code,
                    r.description,
                    r.created_at,
                ))
            })
            .collect()
    }

    async fn assign_role(
        &self,
        user_id: BackofficeUserId,
        role_id: BackofficeRoleId,
    ) -> Result<(), BackofficeIdentityError> {
        sqlx::query(
            r#"
            INSERT INTO backoffice_user_roles (backoffice_user_id, role_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(role_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(BackofficeIdentityError::Database)?;
        Ok(())
    }

    async fn remove_role(
        &self,
        user_id: BackofficeUserId,
        role_id: BackofficeRoleId,
    ) -> Result<(), BackofficeIdentityError> {
        sqlx::query(
            r#"
            DELETE FROM backoffice_user_roles
            WHERE backoffice_user_id = $1 AND role_id = $2
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(role_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(BackofficeIdentityError::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{NoContext, Timestamp, Uuid};

    fn make_user(email: &str) -> BackofficeUser {
        let email = BackofficeEmail::new(email).unwrap();
        BackofficeUser::new(
            BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext))),
            email,
            "argon2_hash_placeholder".to_string(),
            None,
            true,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
    }

    #[test]
    fn test_pg_user_repository_struct_exists() {
        let _type_check: fn(PgPool) -> PgBackofficeUserRepository = PgBackofficeUserRepository::new;
        fn assert_impl<T: BackofficeUserRepository>() {}
        assert_impl::<PgBackofficeUserRepository>();
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn save_then_find_by_id_returns_user(pool: PgPool) {
        let repo = PgBackofficeUserRepository::new(pool);
        let user = make_user("admin@example.com");
        let user_id = *user.id();

        repo.save(&user).await.expect("save should succeed");

        let found = repo
            .find_by_id(user_id)
            .await
            .expect("find_by_id should not error")
            .expect("user should exist");

        assert_eq!(found.id(), &user_id);
        assert_eq!(found.email().as_str(), "admin@example.com");
        assert_eq!(found.password_hash(), "argon2_hash_placeholder");
        assert!(found.mfa_secret().is_none(), "mfa_secret must be None");
        assert!(found.is_active(), "is_active must default to true");
        assert!(found.last_login_at().is_none(), "last_login_at must be None");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn save_then_find_by_email_exact_match(pool: PgPool) {
        let repo = PgBackofficeUserRepository::new(pool);
        let user = make_user("billing@example.com");
        let user_id = *user.id();

        repo.save(&user).await.expect("save should succeed");

        let email = BackofficeEmail::new("billing@example.com").unwrap();
        let found = repo
            .find_by_email(&email)
            .await
            .expect("find_by_email should not error")
            .expect("user should exist by email");

        assert_eq!(found.id(), &user_id);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn find_by_id_missing_returns_none(pool: PgPool) {
        let repo = PgBackofficeUserRepository::new(pool);
        let missing_id = BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));

        let result = repo
            .find_by_id(missing_id)
            .await
            .expect("find_by_id should not error on missing id");

        assert!(result.is_none());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn save_duplicate_email_returns_duplicate_email_error(pool: PgPool) {
        let repo = PgBackofficeUserRepository::new(pool);
        let user1 = make_user("dup@example.com");
        let user2 = BackofficeUser::new(
            BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext))),
            BackofficeEmail::new("dup@example.com").unwrap(),
            "other_hash".to_string(),
            None,
            true,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
        );

        repo.save(&user1).await.expect("first save should succeed");

        let err = repo
            .save(&user2)
            .await
            .expect_err("second save with duplicate email should fail");

        assert!(
            matches!(err, BackofficeIdentityError::DuplicateEmail(_)),
            "expected DuplicateEmail, got: {:?}",
            err
        );
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn update_mutates_existing_user(pool: PgPool) {
        let repo = PgBackofficeUserRepository::new(pool);
        let mut user = make_user("update@example.com");
        let user_id = *user.id();

        repo.save(&user).await.expect("save should succeed");

        user.deactivate();
        user.record_login();
        repo.update(&user).await.expect("update should succeed");

        let found = repo
            .find_by_id(user_id)
            .await
            .expect("find_by_id should not error")
            .expect("user should still exist");

        assert!(!found.is_active(), "is_active should be false after deactivate");
        assert!(found.last_login_at().is_some(), "last_login_at should be set");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn list_permissions_for_user_returns_seeded_super_admin_permissions(pool: PgPool) {
        // The seed migration inserts the super_admin role with all 13 permissions.
        // We create a fresh user and assign it the seeded super_admin role,
        // then verify list_permissions_for_user returns 13 permissions.
        let repo = PgBackofficeUserRepository::new(pool.clone());
        let user = make_user("superadmin_test@example.com");
        let user_id = *user.id();

        repo.save(&user).await.expect("save should succeed");

        // The seeded super_admin role has a known constant UUID from the seed migration.
        let super_admin_role_id =
            BackofficeRoleId::from_uuid(Uuid::parse_str("b0cf0001-0000-7000-8000-000000000001").unwrap());

        repo.assign_role(user_id, super_admin_role_id)
            .await
            .expect("assign_role should succeed");

        let permissions = repo
            .list_permissions_for_user(user_id)
            .await
            .expect("list_permissions_for_user should not error");

        assert_eq!(
            permissions.len(),
            13,
            "super_admin must have all 13 permissions; got: {:?}",
            permissions.iter().map(|p| p.code().as_str()).collect::<Vec<_>>()
        );

        // Verify the 6 required FR-ID-4 permissions are present
        let codes: Vec<&str> = permissions.iter().map(|p| p.code().as_str()).collect();
        assert!(codes.contains(&"platform:org.list"));
        assert!(codes.contains(&"platform:org.suspend"));
        assert!(codes.contains(&"platform:plan.create"));
        assert!(codes.contains(&"platform:subscription.force_cancel"));
        assert!(codes.contains(&"platform:audit.read"));
        assert!(codes.contains(&"platform:user.impersonate"));
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn list_permissions_for_user_with_no_roles_returns_empty(pool: PgPool) {
        let repo = PgBackofficeUserRepository::new(pool);
        let user = make_user("noroles@example.com");
        let user_id = *user.id();

        repo.save(&user).await.expect("save should succeed");

        let permissions = repo
            .list_permissions_for_user(user_id)
            .await
            .expect("list_permissions_for_user should not error");

        assert!(
            permissions.is_empty(),
            "user with no roles must return empty permissions list"
        );
    }
}
