// PostgreSQL StoreRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::{Store, User};
use crate::domain::repositories::StoreRepository;
use crate::domain::value_objects::{Email, StoreId, UserId, Username};
use crate::error::IdentityError;

/// PostgreSQL implementation of StoreRepository
pub struct PgStoreRepository {
    pool: PgPool,
}

impl PgStoreRepository {
    /// Creates a new PgStoreRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StoreRepository for PgStoreRepository {
    async fn save(&self, store: &Store) -> Result<(), IdentityError> {
        sqlx::query(
            r#"
            INSERT INTO stores (id, name, address, is_ecommerce, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(store.id().as_uuid())
        .bind(store.name())
        .bind(store.address())
        .bind(store.is_ecommerce())
        .bind(store.is_active())
        .bind(store.created_at())
        .bind(store.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: StoreId) -> Result<Option<Store>, IdentityError> {
        let row = sqlx::query_as::<_, StoreRow>(
            r#"
            SELECT id, name, address, is_ecommerce, is_active, created_at, updated_at
            FROM stores
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_all(&self) -> Result<Vec<Store>, IdentityError> {
        let rows = sqlx::query_as::<_, StoreRow>(
            r#"
            SELECT id, name, address, is_ecommerce, is_active, created_at, updated_at
            FROM stores
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_active(&self) -> Result<Vec<Store>, IdentityError> {
        let rows = sqlx::query_as::<_, StoreRow>(
            r#"
            SELECT id, name, address, is_ecommerce, is_active, created_at, updated_at
            FROM stores
            WHERE is_active = TRUE
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn update(&self, store: &Store) -> Result<(), IdentityError> {
        let result = sqlx::query(
            r#"
            UPDATE stores
            SET name = $2, address = $3, is_ecommerce = $4, is_active = $5, updated_at = $6
            WHERE id = $1
            "#,
        )
        .bind(store.id().as_uuid())
        .bind(store.name())
        .bind(store.address())
        .bind(store.is_ecommerce())
        .bind(store.is_active())
        .bind(store.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(IdentityError::StoreNotFound(store.id().into_uuid()));
        }

        Ok(())
    }

    async fn get_users(&self, store_id: StoreId) -> Result<Vec<User>, IdentityError> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT u.id, u.username, u.email, u.first_name, u.last_name, u.password_hash, u.is_active, u.created_at, u.updated_at
            FROM users u
            INNER JOIN user_stores us ON u.id = us.user_id
            WHERE us.store_id = $1
            ORDER BY u.username
            "#,
        )
        .bind(store_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
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
