use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::BookingError;
use crate::domain::entities::Resource;
use crate::domain::repositories::ResourceRepository;
use crate::domain::value_objects::{ResourceId, ResourceType};

pub struct PgResourceRepository {
    pool: PgPool,
}

impl PgResourceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ResourceRepository for PgResourceRepository {
    async fn save(&self, r: &Resource) -> Result<(), BookingError> {
        sqlx::query(
            r#"
            INSERT INTO booking_resources (
                id, store_id, resource_type, name, color,
                is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(r.id().into_uuid())
        .bind(r.store_id())
        .bind(r.resource_type().as_str())
        .bind(r.name())
        .bind(r.color())
        .bind(r.is_active())
        .bind(r.created_at())
        .bind(r.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, r: &Resource) -> Result<(), BookingError> {
        let result = sqlx::query(
            r#"
            UPDATE booking_resources
               SET name       = $2,
                   color      = $3,
                   is_active  = $4,
                   updated_at = $5
             WHERE id = $1
            "#,
        )
        .bind(r.id().into_uuid())
        .bind(r.name())
        .bind(r.color())
        .bind(r.is_active())
        .bind(r.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(BookingError::ResourceNotFound(r.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(&self, id: ResourceId) -> Result<Option<Resource>, BookingError> {
        let row = sqlx::query_as::<_, ResourceRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(Resource::try_from).transpose()
    }

    async fn list_by_store(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<Resource>, BookingError> {
        let sql = if only_active {
            LIST_BY_STORE_ACTIVE
        } else {
            LIST_BY_STORE_ALL
        };
        let rows = sqlx::query_as::<_, ResourceRow>(sql)
            .bind(store_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Resource::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, store_id, resource_type, name, color, is_active, created_at, updated_at
FROM booking_resources
WHERE id = $1
"#;

const LIST_BY_STORE_ACTIVE: &str = r#"
SELECT id, store_id, resource_type, name, color, is_active, created_at, updated_at
FROM booking_resources
WHERE store_id = $1 AND is_active = TRUE
ORDER BY name
"#;

const LIST_BY_STORE_ALL: &str = r#"
SELECT id, store_id, resource_type, name, color, is_active, created_at, updated_at
FROM booking_resources
WHERE store_id = $1
ORDER BY name
"#;

#[derive(sqlx::FromRow)]
struct ResourceRow {
    id: Uuid,
    store_id: Uuid,
    resource_type: String,
    name: String,
    color: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ResourceRow> for Resource {
    type Error = BookingError;
    fn try_from(row: ResourceRow) -> Result<Self, BookingError> {
        Ok(Resource::reconstitute(
            ResourceId::from_uuid(row.id),
            row.store_id,
            ResourceType::from_str(&row.resource_type)?,
            row.name,
            row.color,
            row.is_active,
            row.created_at,
            row.updated_at,
        ))
    }
}
