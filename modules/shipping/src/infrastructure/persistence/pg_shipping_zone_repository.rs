use async_trait::async_trait;
use sqlx::PgPool;

use crate::ShippingError;
use crate::domain::entities::ShippingZone;
use crate::domain::repositories::ShippingZoneRepository;
use crate::domain::value_objects::ShippingZoneId;
use identity::StoreId;

pub struct PgShippingZoneRepository {
    pool: PgPool,
}

impl PgShippingZoneRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShippingZoneRepository for PgShippingZoneRepository {
    async fn save(&self, z: &ShippingZone) -> Result<(), ShippingError> {
        sqlx::query(
            r#"INSERT INTO shipping_zones
              (id, store_id, name, countries, states, zip_codes, is_active, created_at, updated_at)
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"#,
        )
        .bind(z.id().into_uuid())
        .bind(z.store_id().into_uuid())
        .bind(z.name())
        .bind(z.countries())
        .bind(z.states())
        .bind(z.zip_codes())
        .bind(z.is_active())
        .bind(z.created_at())
        .bind(z.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: ShippingZoneId) -> Result<Option<ShippingZone>, ShippingError> {
        let row = sqlx::query_as::<_, ZoneRow>(SELECT_SQL)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<ShippingZone>, ShippingError> {
        let rows = sqlx::query_as::<_, ZoneRow>(
            r#"SELECT id, store_id, name, countries, states, zip_codes,
                   is_active, created_at, updated_at
               FROM shipping_zones WHERE store_id = $1 ORDER BY name"#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_active_by_store(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<ShippingZone>, ShippingError> {
        let rows = sqlx::query_as::<_, ZoneRow>(
            r#"SELECT id, store_id, name, countries, states, zip_codes,
                   is_active, created_at, updated_at
               FROM shipping_zones
               WHERE store_id = $1 AND is_active = true ORDER BY name"#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&self, z: &ShippingZone) -> Result<(), ShippingError> {
        let result = sqlx::query(
            r#"UPDATE shipping_zones
               SET name=$2, countries=$3, states=$4, zip_codes=$5,
                   is_active=$6, updated_at=$7
               WHERE id=$1"#,
        )
        .bind(z.id().into_uuid())
        .bind(z.name())
        .bind(z.countries())
        .bind(z.states())
        .bind(z.zip_codes())
        .bind(z.is_active())
        .bind(z.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::ShippingZoneNotFound(z.id().into_uuid()));
        }
        Ok(())
    }

    async fn delete(&self, id: ShippingZoneId) -> Result<(), ShippingError> {
        let result = sqlx::query("DELETE FROM shipping_zones WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::ShippingZoneNotFound(id.into_uuid()));
        }
        Ok(())
    }
}

const SELECT_SQL: &str = r#"
SELECT id, store_id, name, countries, states, zip_codes,
       is_active, created_at, updated_at
FROM shipping_zones WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct ZoneRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    name: String,
    countries: Vec<String>,
    states: Vec<String>,
    zip_codes: Vec<String>,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ZoneRow> for ShippingZone {
    fn from(row: ZoneRow) -> Self {
        ShippingZone::reconstitute(
            ShippingZoneId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            row.name,
            row.countries,
            row.states,
            row.zip_codes,
            row.is_active,
            row.created_at,
            row.updated_at,
        )
    }
}
