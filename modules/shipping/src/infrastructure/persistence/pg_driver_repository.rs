use async_trait::async_trait;
use sqlx::PgPool;

use crate::ShippingError;
use crate::domain::entities::Driver;
use crate::domain::repositories::DriverRepository;
use crate::domain::value_objects::{DriverId, DriverStatus, VehicleType};
use identity::{StoreId, UserId};

pub struct PgDriverRepository {
    pool: PgPool,
}

impl PgDriverRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriverRepository for PgDriverRepository {
    async fn save(&self, d: &Driver) -> Result<(), ShippingError> {
        sqlx::query(
            r#"INSERT INTO drivers
              (id, store_id, user_id, name, phone, vehicle_type, license_plate,
               is_active, current_status, created_at, updated_at)
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#,
        )
        .bind(d.id().into_uuid())
        .bind(d.store_id().into_uuid())
        .bind(d.user_id().map(|u| u.into_uuid()))
        .bind(d.name())
        .bind(d.phone())
        .bind(d.vehicle_type().to_string())
        .bind(d.license_plate())
        .bind(d.is_active())
        .bind(d.current_status().to_string())
        .bind(d.created_at())
        .bind(d.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: DriverId) -> Result<Option<Driver>, ShippingError> {
        let row = sqlx::query_as::<_, DriverRow>(SELECT_SQL)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<Driver>, ShippingError> {
        let rows = sqlx::query_as::<_, DriverRow>(
            r#"SELECT id, store_id, user_id, name, phone, vehicle_type, license_plate,
                   is_active, current_status, created_at, updated_at
               FROM drivers WHERE store_id = $1 ORDER BY name"#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_available_by_store(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<Driver>, ShippingError> {
        let rows = sqlx::query_as::<_, DriverRow>(
            r#"SELECT id, store_id, user_id, name, phone, vehicle_type, license_plate,
                   is_active, current_status, created_at, updated_at
               FROM drivers
               WHERE store_id = $1 AND is_active = true AND current_status = 'available'
               ORDER BY name"#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn update(&self, d: &Driver) -> Result<(), ShippingError> {
        let result = sqlx::query(
            r#"UPDATE drivers
               SET name=$2, phone=$3, vehicle_type=$4, license_plate=$5,
                   is_active=$6, current_status=$7, updated_at=$8
               WHERE id=$1"#,
        )
        .bind(d.id().into_uuid())
        .bind(d.name())
        .bind(d.phone())
        .bind(d.vehicle_type().to_string())
        .bind(d.license_plate())
        .bind(d.is_active())
        .bind(d.current_status().to_string())
        .bind(d.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::DriverNotFound(d.id().into_uuid()));
        }
        Ok(())
    }

    async fn delete(&self, id: DriverId) -> Result<(), ShippingError> {
        let result = sqlx::query("DELETE FROM drivers WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::DriverNotFound(id.into_uuid()));
        }
        Ok(())
    }
}

const SELECT_SQL: &str = r#"
SELECT id, store_id, user_id, name, phone, vehicle_type, license_plate,
       is_active, current_status, created_at, updated_at
FROM drivers WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct DriverRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    user_id: Option<uuid::Uuid>,
    name: String,
    phone: String,
    vehicle_type: String,
    license_plate: Option<String>,
    is_active: bool,
    current_status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<DriverRow> for Driver {
    type Error = ShippingError;
    fn try_from(row: DriverRow) -> Result<Self, Self::Error> {
        let vehicle_type: VehicleType = row
            .vehicle_type
            .parse()
            .map_err(|_| ShippingError::InvalidVehicleType)?;
        let current_status: DriverStatus = row
            .current_status
            .parse()
            .map_err(|_| ShippingError::InvalidDriverStatus)?;
        Ok(Driver::reconstitute(
            DriverId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            row.user_id.map(UserId::from_uuid),
            row.name,
            row.phone,
            vehicle_type,
            row.license_plate,
            row.is_active,
            current_status,
            row.created_at,
            row.updated_at,
        ))
    }
}
