use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::entities::KitchenStation;
use crate::domain::repositories::KitchenStationRepository;
use crate::domain::value_objects::KitchenStationId;

pub struct PgKitchenStationRepository {
    pool: PgPool,
}

impl PgKitchenStationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl KitchenStationRepository for PgKitchenStationRepository {
    async fn save(&self, s: &KitchenStation) -> Result<(), RestaurantOperationsError> {
        sqlx::query(
            r#"
            INSERT INTO kitchen_stations (
                id, store_id, name, color, sort_order, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(s.id().into_uuid())
        .bind(s.store_id())
        .bind(s.name())
        .bind(s.color())
        .bind(s.sort_order())
        .bind(s.is_active())
        .bind(s.created_at())
        .bind(s.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, s: &KitchenStation) -> Result<(), RestaurantOperationsError> {
        let result = sqlx::query(
            r#"
            UPDATE kitchen_stations
               SET name       = $2,
                   color      = $3,
                   sort_order = $4,
                   is_active  = $5,
                   updated_at = $6
             WHERE id = $1
            "#,
        )
        .bind(s.id().into_uuid())
        .bind(s.name())
        .bind(s.color())
        .bind(s.sort_order())
        .bind(s.is_active())
        .bind(s.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RestaurantOperationsError::StationNotFound(
                s.id().into_uuid(),
            ));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: KitchenStationId,
    ) -> Result<Option<KitchenStation>, RestaurantOperationsError> {
        let row = sqlx::query_as::<_, StationRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(KitchenStation::from))
    }

    async fn list_by_store(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<KitchenStation>, RestaurantOperationsError> {
        let sql = if only_active { LIST_ACTIVE } else { LIST_ALL };
        let rows = sqlx::query_as::<_, StationRow>(sql)
            .bind(store_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(KitchenStation::from).collect())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, store_id, name, color, sort_order, is_active, created_at, updated_at
FROM kitchen_stations
WHERE id = $1
"#;

const LIST_ACTIVE: &str = r#"
SELECT id, store_id, name, color, sort_order, is_active, created_at, updated_at
FROM kitchen_stations
WHERE store_id = $1 AND is_active = TRUE
ORDER BY sort_order, name
"#;

const LIST_ALL: &str = r#"
SELECT id, store_id, name, color, sort_order, is_active, created_at, updated_at
FROM kitchen_stations
WHERE store_id = $1
ORDER BY sort_order, name
"#;

#[derive(sqlx::FromRow)]
struct StationRow {
    id: Uuid,
    store_id: Uuid,
    name: String,
    color: Option<String>,
    sort_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<StationRow> for KitchenStation {
    fn from(r: StationRow) -> Self {
        KitchenStation::reconstitute(
            KitchenStationId::from_uuid(r.id),
            r.store_id,
            r.name,
            r.color,
            r.sort_order,
            r.is_active,
            r.created_at,
            r.updated_at,
        )
    }
}
