// PostgreSQL UserRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::{Permission, Role, Store, User};
use crate::domain::repositories::UserRepository;
use crate::domain::value_objects::{
    Email, PermissionCode, PermissionId, RoleId, StoreId, UserId, Username,
};
use crate::error::IdentityError;

/// PostgreSQL implementation of UserRepository
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    /// Creates a new PgUserRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn save(&self, user: &User) -> Result<(), IdentityError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, first_name, last_name, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(user.id().as_uuid())
        .bind(user.username().as_str())
        .bind(user.email().as_str())
        .bind(user.first_name())
        .bind(user.last_name())
        .bind(user.password_hash())
        .bind(user.is_active())
        .bind(user.created_at())
        .bind(user.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                let constraint = db_err.constraint().unwrap_or("");
                if constraint.contains("username") {
                    IdentityError::DuplicateUsername(user.username().as_str().to_string())
                } else if constraint.contains("email") {
                    IdentityError::DuplicateEmail(user.email().as_str().to_string())
                } else {
                    IdentityError::Database(e)
                }
            }
            _ => IdentityError::Database(e),
        })?;

        Ok(())
    }


    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, IdentityError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, first_name, last_name, password_hash, is_active, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, IdentityError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, first_name, last_name, password_hash, is_active, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_username(&self, username: &Username) -> Result<Option<User>, IdentityError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, email, first_name, last_name, password_hash, is_active, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn update(&self, user: &User) -> Result<(), IdentityError> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET username = $2, email = $3, first_name = $4, last_name = $5, 
                password_hash = $6, is_active = $7, updated_at = $8
            WHERE id = $1
            "#,
        )
        .bind(user.id().as_uuid())
        .bind(user.username().as_str())
        .bind(user.email().as_str())
        .bind(user.first_name())
        .bind(user.last_name())
        .bind(user.password_hash())
        .bind(user.is_active())
        .bind(user.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                IdentityError::DuplicateEmail(user.email().as_str().to_string())
            }
            _ => IdentityError::Database(e),
        })?;

        if result.rows_affected() == 0 {
            return Err(IdentityError::UserNotFound(user.id().into_uuid()));
        }

        Ok(())
    }


    async fn assign_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        store_id: StoreId,
    ) -> Result<(), IdentityError> {
        // Verify user exists
        let user_exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"#,
        )
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        if !user_exists {
            return Err(IdentityError::UserNotFound(user_id.into_uuid()));
        }

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

        // Verify store exists
        let store_exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM stores WHERE id = $1)"#,
        )
        .bind(store_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        if !store_exists {
            return Err(IdentityError::StoreNotFound(store_id.into_uuid()));
        }

        // Verify user is member of store
        let is_member = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM user_stores WHERE user_id = $1 AND store_id = $2)"#,
        )
        .bind(user_id.as_uuid())
        .bind(store_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        if !is_member {
            return Err(IdentityError::UserNotInStore(store_id.into_uuid()));
        }

        // Insert the role assignment (ignore if already exists)
        sqlx::query(
            r#"
            INSERT INTO user_store_roles (user_id, store_id, role_id, created_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (user_id, store_id, role_id) DO NOTHING
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(store_id.as_uuid())
        .bind(role_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        store_id: StoreId,
    ) -> Result<(), IdentityError> {
        // Verify user exists
        let user_exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"#,
        )
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        if !user_exists {
            return Err(IdentityError::UserNotFound(user_id.into_uuid()));
        }

        sqlx::query(
            r#"
            DELETE FROM user_store_roles
            WHERE user_id = $1 AND store_id = $2 AND role_id = $3
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(store_id.as_uuid())
        .bind(role_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }


    async fn get_roles_for_store(
        &self,
        user_id: UserId,
        store_id: StoreId,
    ) -> Result<Vec<Role>, IdentityError> {
        let rows = sqlx::query_as::<_, RoleRow>(
            r#"
            SELECT r.id, r.name, r.description, r.is_system_protected, r.created_at, r.updated_at
            FROM roles r
            INNER JOIN user_store_roles usr ON r.id = usr.role_id
            WHERE usr.user_id = $1 AND usr.store_id = $2
            ORDER BY r.name
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(store_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn get_permissions_for_store(
        &self,
        user_id: UserId,
        store_id: StoreId,
    ) -> Result<Vec<Permission>, IdentityError> {
        // This query returns permissions from:
        // 1. Roles assigned to the user in the specific store
        // 2. The super_admin role if the user has it in ANY store (global permissions)
        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT DISTINCT p.id, p.code, p.description, p.created_at
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            INNER JOIN user_store_roles usr ON rp.role_id = usr.role_id
            WHERE usr.user_id = $1 
              AND (
                  usr.store_id = $2
                  OR EXISTS (
                      SELECT 1 FROM roles r 
                      WHERE r.id = usr.role_id 
                        AND r.name = 'super_admin'
                  )
              )
            ORDER BY p.code
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(store_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn remove_role_from_all_users(&self, role_id: RoleId) -> Result<(), IdentityError> {
        sqlx::query(
            r#"
            DELETE FROM user_store_roles
            WHERE role_id = $1
            "#,
        )
        .bind(role_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn add_to_store(&self, user_id: UserId, store_id: StoreId) -> Result<(), IdentityError> {
        // Verify user exists
        let user_exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"#,
        )
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        if !user_exists {
            return Err(IdentityError::UserNotFound(user_id.into_uuid()));
        }

        // Verify store exists
        let store_exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM stores WHERE id = $1)"#,
        )
        .bind(store_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        if !store_exists {
            return Err(IdentityError::StoreNotFound(store_id.into_uuid()));
        }

        // Insert the membership (ignore if already exists)
        sqlx::query(
            r#"
            INSERT INTO user_stores (user_id, store_id, created_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (user_id, store_id) DO NOTHING
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(store_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }


    async fn remove_from_store(
        &self,
        user_id: UserId,
        store_id: StoreId,
    ) -> Result<(), IdentityError> {
        // Verify user exists
        let user_exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"#,
        )
        .bind(user_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        if !user_exists {
            return Err(IdentityError::UserNotFound(user_id.into_uuid()));
        }

        // Delete membership (cascades will handle user_store_roles)
        sqlx::query(
            r#"
            DELETE FROM user_stores
            WHERE user_id = $1 AND store_id = $2
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(store_id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_stores(&self, user_id: UserId) -> Result<Vec<Store>, IdentityError> {
        let rows = sqlx::query_as::<_, StoreRow>(
            r#"
            SELECT s.id, s.name, s.address, s.is_ecommerce, s.is_active, s.created_at, s.updated_at
            FROM stores s
            INNER JOIN user_stores us ON s.id = us.store_id
            WHERE us.user_id = $1
            ORDER BY s.name
            "#,
        )
        .bind(user_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn is_member_of_store(
        &self,
        user_id: UserId,
        store_id: StoreId,
    ) -> Result<bool, IdentityError> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(SELECT 1 FROM user_stores WHERE user_id = $1 AND store_id = $2)
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(store_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }
}

/// Internal row type for mapping user database results
#[derive(sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    username: String,
    email: String,
    first_name: String,
    last_name: String,
    password_hash: String,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = IdentityError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let username = Username::new(&row.username)?;
        let email = Email::new(&row.email)?;
        Ok(User::new(
            UserId::from_uuid(row.id),
            username,
            email,
            row.first_name,
            row.last_name,
            row.password_hash,
            row.is_active,
            row.created_at,
            row.updated_at,
        ))
    }
}

/// Internal row type for mapping role database results
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

/// Internal row type for mapping permission database results
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

/// Internal row type for mapping store database results
#[derive(sqlx::FromRow)]
struct StoreRow {
    id: uuid::Uuid,
    name: String,
    address: String,
    is_ecommerce: bool,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<StoreRow> for Store {
    fn from(row: StoreRow) -> Self {
        Store::new(
            StoreId::from_uuid(row.id),
            row.name,
            row.address,
            row.is_ecommerce,
            row.is_active,
            row.created_at,
            row.updated_at,
        )
    }
}
