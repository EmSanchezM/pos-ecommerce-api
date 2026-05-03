use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::BookingError;
use crate::domain::entities::Service;
use crate::domain::repositories::ServiceRepository;
use crate::domain::value_objects::{ResourceId, ServiceId};

pub struct PgServiceRepository {
    pool: PgPool,
}

impl PgServiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ServiceRepository for PgServiceRepository {
    async fn save(&self, s: &Service) -> Result<(), BookingError> {
        sqlx::query(
            r#"
            INSERT INTO booking_services (
                id, store_id, name, description, duration_minutes, price,
                buffer_minutes_before, buffer_minutes_after,
                requires_deposit, deposit_amount,
                is_active, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
            )
            "#,
        )
        .bind(s.id().into_uuid())
        .bind(s.store_id())
        .bind(s.name())
        .bind(s.description())
        .bind(s.duration_minutes())
        .bind(s.price())
        .bind(s.buffer_minutes_before())
        .bind(s.buffer_minutes_after())
        .bind(s.requires_deposit())
        .bind(s.deposit_amount())
        .bind(s.is_active())
        .bind(s.created_at())
        .bind(s.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, s: &Service) -> Result<(), BookingError> {
        let result = sqlx::query(
            r#"
            UPDATE booking_services
               SET name                  = $2,
                   description           = $3,
                   duration_minutes      = $4,
                   price                 = $5,
                   buffer_minutes_before = $6,
                   buffer_minutes_after  = $7,
                   requires_deposit      = $8,
                   deposit_amount        = $9,
                   is_active             = $10,
                   updated_at            = $11
             WHERE id = $1
            "#,
        )
        .bind(s.id().into_uuid())
        .bind(s.name())
        .bind(s.description())
        .bind(s.duration_minutes())
        .bind(s.price())
        .bind(s.buffer_minutes_before())
        .bind(s.buffer_minutes_after())
        .bind(s.requires_deposit())
        .bind(s.deposit_amount())
        .bind(s.is_active())
        .bind(s.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(BookingError::ServiceNotFound(s.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(&self, id: ServiceId) -> Result<Option<Service>, BookingError> {
        let row = sqlx::query_as::<_, ServiceRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Service::from))
    }

    async fn list_by_store(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<Service>, BookingError> {
        let sql = if only_active { LIST_ACTIVE } else { LIST_ALL };
        let rows = sqlx::query_as::<_, ServiceRow>(sql)
            .bind(store_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(Service::from).collect())
    }

    async fn assign_resources(
        &self,
        service_id: ServiceId,
        resource_ids: &[ResourceId],
    ) -> Result<(), BookingError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM booking_service_resources WHERE service_id = $1")
            .bind(service_id.into_uuid())
            .execute(&mut *tx)
            .await?;
        for rid in resource_ids {
            sqlx::query(
                r#"
                INSERT INTO booking_service_resources (service_id, resource_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(service_id.into_uuid())
            .bind(rid.into_uuid())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn find_eligible_resources(
        &self,
        service_id: ServiceId,
    ) -> Result<Vec<ResourceId>, BookingError> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT resource_id
            FROM booking_service_resources
            WHERE service_id = $1
            "#,
        )
        .bind(service_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id,)| ResourceId::from_uuid(id))
            .collect())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, store_id, name, description, duration_minutes, price,
       buffer_minutes_before, buffer_minutes_after,
       requires_deposit, deposit_amount,
       is_active, created_at, updated_at
FROM booking_services
WHERE id = $1
"#;

const LIST_ACTIVE: &str = r#"
SELECT id, store_id, name, description, duration_minutes, price,
       buffer_minutes_before, buffer_minutes_after,
       requires_deposit, deposit_amount,
       is_active, created_at, updated_at
FROM booking_services
WHERE store_id = $1 AND is_active = TRUE
ORDER BY name
"#;

const LIST_ALL: &str = r#"
SELECT id, store_id, name, description, duration_minutes, price,
       buffer_minutes_before, buffer_minutes_after,
       requires_deposit, deposit_amount,
       is_active, created_at, updated_at
FROM booking_services
WHERE store_id = $1
ORDER BY name
"#;

#[derive(sqlx::FromRow)]
struct ServiceRow {
    id: Uuid,
    store_id: Uuid,
    name: String,
    description: Option<String>,
    duration_minutes: i32,
    price: Decimal,
    buffer_minutes_before: i32,
    buffer_minutes_after: i32,
    requires_deposit: bool,
    deposit_amount: Option<Decimal>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ServiceRow> for Service {
    fn from(r: ServiceRow) -> Self {
        Service::reconstitute(
            ServiceId::from_uuid(r.id),
            r.store_id,
            r.name,
            r.description,
            r.duration_minutes,
            r.price,
            r.buffer_minutes_before,
            r.buffer_minutes_after,
            r.requires_deposit,
            r.deposit_amount,
            r.is_active,
            r.created_at,
            r.updated_at,
        )
    }
}
