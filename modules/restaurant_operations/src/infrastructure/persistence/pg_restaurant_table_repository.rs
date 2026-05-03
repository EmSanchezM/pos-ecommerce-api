use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::entities::RestaurantTable;
use crate::domain::repositories::RestaurantTableRepository;
use crate::domain::value_objects::{RestaurantTableId, TableStatus};

pub struct PgRestaurantTableRepository {
    pool: PgPool,
}

impl PgRestaurantTableRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RestaurantTableRepository for PgRestaurantTableRepository {
    async fn save(&self, t: &RestaurantTable) -> Result<(), RestaurantOperationsError> {
        sqlx::query(
            r#"
            INSERT INTO restaurant_tables (
                id, store_id, label, capacity, status,
                current_ticket_id, notes, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(t.id().into_uuid())
        .bind(t.store_id())
        .bind(t.label())
        .bind(t.capacity())
        .bind(t.status().as_str())
        .bind(t.current_ticket_id())
        .bind(t.notes())
        .bind(t.is_active())
        .bind(t.created_at())
        .bind(t.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, t: &RestaurantTable) -> Result<(), RestaurantOperationsError> {
        let result = sqlx::query(
            r#"
            UPDATE restaurant_tables
               SET label             = $2,
                   capacity          = $3,
                   status            = $4,
                   current_ticket_id = $5,
                   notes             = $6,
                   is_active         = $7,
                   updated_at        = $8
             WHERE id = $1
            "#,
        )
        .bind(t.id().into_uuid())
        .bind(t.label())
        .bind(t.capacity())
        .bind(t.status().as_str())
        .bind(t.current_ticket_id())
        .bind(t.notes())
        .bind(t.is_active())
        .bind(t.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RestaurantOperationsError::TableNotFound(t.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: RestaurantTableId,
    ) -> Result<Option<RestaurantTable>, RestaurantOperationsError> {
        let row = sqlx::query_as::<_, TableRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(RestaurantTable::try_from).transpose()
    }

    async fn list_by_store(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<RestaurantTable>, RestaurantOperationsError> {
        let sql = if only_active { LIST_ACTIVE } else { LIST_ALL };
        let rows = sqlx::query_as::<_, TableRow>(sql)
            .bind(store_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(RestaurantTable::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, store_id, label, capacity, status,
       current_ticket_id, notes, is_active, created_at, updated_at
FROM restaurant_tables
WHERE id = $1
"#;

const LIST_ACTIVE: &str = r#"
SELECT id, store_id, label, capacity, status,
       current_ticket_id, notes, is_active, created_at, updated_at
FROM restaurant_tables
WHERE store_id = $1 AND is_active = TRUE
ORDER BY label
"#;

const LIST_ALL: &str = r#"
SELECT id, store_id, label, capacity, status,
       current_ticket_id, notes, is_active, created_at, updated_at
FROM restaurant_tables
WHERE store_id = $1
ORDER BY label
"#;

#[derive(sqlx::FromRow)]
struct TableRow {
    id: Uuid,
    store_id: Uuid,
    label: String,
    capacity: i32,
    status: String,
    current_ticket_id: Option<Uuid>,
    notes: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TableRow> for RestaurantTable {
    type Error = RestaurantOperationsError;
    fn try_from(r: TableRow) -> Result<Self, RestaurantOperationsError> {
        Ok(RestaurantTable::reconstitute(
            RestaurantTableId::from_uuid(r.id),
            r.store_id,
            r.label,
            r.capacity,
            TableStatus::from_str(&r.status)?,
            r.current_ticket_id,
            r.notes,
            r.is_active,
            r.created_at,
            r.updated_at,
        ))
    }
}
