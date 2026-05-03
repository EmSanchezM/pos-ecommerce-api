//! PgAppointmentRepository — note `save_with_slot_check` runs the conflict
//! query under `FOR UPDATE` inside a transaction so two concurrent public
//! bookings against the same `(resource_id, time window)` cannot both insert.

use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::BookingError;
use crate::domain::entities::Appointment;
use crate::domain::repositories::{AppointmentRepository, ListAppointmentsFilters};
use crate::domain::value_objects::{AppointmentId, AppointmentStatus, ResourceId, ServiceId};

pub struct PgAppointmentRepository {
    pool: PgPool,
}

impl PgAppointmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AppointmentRepository for PgAppointmentRepository {
    async fn save_with_slot_check(&self, a: &Appointment) -> Result<(), BookingError> {
        let mut tx = self.pool.begin().await?;

        // Lock overlapping appointments for the same resource that are still
        // occupying their slot. If any row exists, the slot is taken.
        let conflict: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id
              FROM booking_appointments
             WHERE resource_id = $1
               AND status IN ('scheduled', 'confirmed', 'in_progress')
               AND tstzrange(starts_at, ends_at, '[)') &&
                   tstzrange($2, $3, '[)')
             FOR UPDATE
             LIMIT 1
            "#,
        )
        .bind(a.resource_id().into_uuid())
        .bind(a.starts_at())
        .bind(a.ends_at())
        .fetch_optional(&mut *tx)
        .await?;

        if conflict.is_some() {
            return Err(BookingError::SlotConflict {
                start: a.starts_at(),
                end: a.ends_at(),
            });
        }

        sqlx::query(
            r#"
            INSERT INTO booking_appointments (
                id, store_id, service_id, resource_id, customer_id,
                customer_name, customer_email, customer_phone,
                starts_at, ends_at, status,
                deposit_transaction_id, generated_sale_id,
                notes, canceled_reason, no_show_at,
                public_token, created_by, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16, $17, $18, $19, $20
            )
            "#,
        )
        .bind(a.id().into_uuid())
        .bind(a.store_id())
        .bind(a.service_id().into_uuid())
        .bind(a.resource_id().into_uuid())
        .bind(a.customer_id())
        .bind(a.customer_name())
        .bind(a.customer_email())
        .bind(a.customer_phone())
        .bind(a.starts_at())
        .bind(a.ends_at())
        .bind(a.status().as_str())
        .bind(a.deposit_transaction_id())
        .bind(a.generated_sale_id())
        .bind(a.notes())
        .bind(a.canceled_reason())
        .bind(a.no_show_at())
        .bind(a.public_token())
        .bind(a.created_by())
        .bind(a.created_at())
        .bind(a.updated_at())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn find_by_id(&self, id: AppointmentId) -> Result<Option<Appointment>, BookingError> {
        let row = sqlx::query_as::<_, AppointmentRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(Appointment::try_from).transpose()
    }

    async fn find_by_public_token(&self, token: &str) -> Result<Option<Appointment>, BookingError> {
        let row = sqlx::query_as::<_, AppointmentRow>(SELECT_BY_TOKEN)
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Appointment::try_from).transpose()
    }

    async fn list_occupying_slots(
        &self,
        resource_id: ResourceId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Appointment>, BookingError> {
        let rows = sqlx::query_as::<_, AppointmentRow>(LIST_OCCUPYING)
            .bind(resource_id.into_uuid())
            .bind(from)
            .bind(to)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Appointment::try_from).collect()
    }

    async fn list(
        &self,
        filters: ListAppointmentsFilters,
    ) -> Result<Vec<Appointment>, BookingError> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, store_id, service_id, resource_id, customer_id, \
             customer_name, customer_email, customer_phone, \
             starts_at, ends_at, status, deposit_transaction_id, generated_sale_id, \
             notes, canceled_reason, no_show_at, public_token, \
             created_by, created_at, updated_at \
             FROM booking_appointments WHERE 1=1",
        );
        if let Some(store_id) = filters.store_id {
            qb.push(" AND store_id = ").push_bind(store_id);
        }
        if let Some(resource_id) = filters.resource_id {
            qb.push(" AND resource_id = ")
                .push_bind(resource_id.into_uuid());
        }
        if let Some(customer_id) = filters.customer_id {
            qb.push(" AND customer_id = ").push_bind(customer_id);
        }
        if let Some(status) = filters.status {
            qb.push(" AND status = ").push_bind(status.as_str());
        }
        if let Some(from) = filters.from {
            qb.push(" AND starts_at >= ").push_bind(from);
        }
        if let Some(to) = filters.to {
            qb.push(" AND starts_at < ").push_bind(to);
        }
        qb.push(" ORDER BY starts_at DESC");
        if let Some(limit) = filters.limit {
            qb.push(" LIMIT ").push_bind(limit);
        } else {
            qb.push(" LIMIT 200");
        }
        let rows: Vec<AppointmentRow> = qb.build_query_as().fetch_all(&self.pool).await?;
        rows.into_iter().map(Appointment::try_from).collect()
    }

    async fn update(&self, a: &Appointment) -> Result<(), BookingError> {
        let result = sqlx::query(
            r#"
            UPDATE booking_appointments
               SET status                  = $2,
                   deposit_transaction_id  = $3,
                   generated_sale_id       = $4,
                   notes                   = $5,
                   canceled_reason         = $6,
                   no_show_at              = $7,
                   updated_at              = $8
             WHERE id = $1
            "#,
        )
        .bind(a.id().into_uuid())
        .bind(a.status().as_str())
        .bind(a.deposit_transaction_id())
        .bind(a.generated_sale_id())
        .bind(a.notes())
        .bind(a.canceled_reason())
        .bind(a.no_show_at())
        .bind(a.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(BookingError::AppointmentNotFound(a.id().into_uuid()));
        }
        Ok(())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, store_id, service_id, resource_id, customer_id,
       customer_name, customer_email, customer_phone,
       starts_at, ends_at, status,
       deposit_transaction_id, generated_sale_id,
       notes, canceled_reason, no_show_at,
       public_token, created_by, created_at, updated_at
FROM booking_appointments
WHERE id = $1
"#;

const SELECT_BY_TOKEN: &str = r#"
SELECT id, store_id, service_id, resource_id, customer_id,
       customer_name, customer_email, customer_phone,
       starts_at, ends_at, status,
       deposit_transaction_id, generated_sale_id,
       notes, canceled_reason, no_show_at,
       public_token, created_by, created_at, updated_at
FROM booking_appointments
WHERE public_token = $1
"#;

const LIST_OCCUPYING: &str = r#"
SELECT id, store_id, service_id, resource_id, customer_id,
       customer_name, customer_email, customer_phone,
       starts_at, ends_at, status,
       deposit_transaction_id, generated_sale_id,
       notes, canceled_reason, no_show_at,
       public_token, created_by, created_at, updated_at
FROM booking_appointments
WHERE resource_id = $1
  AND status IN ('scheduled', 'confirmed', 'in_progress')
  AND starts_at < $3
  AND ends_at   > $2
ORDER BY starts_at
"#;

#[derive(sqlx::FromRow)]
struct AppointmentRow {
    id: Uuid,
    store_id: Uuid,
    service_id: Uuid,
    resource_id: Uuid,
    customer_id: Option<Uuid>,
    customer_name: String,
    customer_email: String,
    customer_phone: Option<String>,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    status: String,
    deposit_transaction_id: Option<Uuid>,
    generated_sale_id: Option<Uuid>,
    notes: Option<String>,
    canceled_reason: Option<String>,
    no_show_at: Option<DateTime<Utc>>,
    public_token: String,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<AppointmentRow> for Appointment {
    type Error = BookingError;
    fn try_from(r: AppointmentRow) -> Result<Self, BookingError> {
        Ok(Appointment::reconstitute(
            AppointmentId::from_uuid(r.id),
            r.store_id,
            ServiceId::from_uuid(r.service_id),
            ResourceId::from_uuid(r.resource_id),
            r.customer_id,
            r.customer_name,
            r.customer_email,
            r.customer_phone,
            r.starts_at,
            r.ends_at,
            AppointmentStatus::from_str(&r.status)?,
            r.deposit_transaction_id,
            r.generated_sale_id,
            r.notes,
            r.canceled_reason,
            r.no_show_at,
            r.public_token,
            r.created_by,
            r.created_at,
            r.updated_at,
        ))
    }
}
