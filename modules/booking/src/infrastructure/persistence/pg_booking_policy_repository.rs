use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::BookingError;
use crate::domain::entities::BookingPolicy;
use crate::domain::repositories::BookingPolicyRepository;
use crate::domain::value_objects::BookingPolicyId;

pub struct PgBookingPolicyRepository {
    pool: PgPool,
}

impl PgBookingPolicyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookingPolicyRepository for PgBookingPolicyRepository {
    async fn upsert(&self, p: &BookingPolicy) -> Result<(), BookingError> {
        sqlx::query(
            r#"
            INSERT INTO booking_policies (
                id, store_id, requires_deposit, deposit_percentage,
                cancellation_window_hours, no_show_fee_amount,
                default_buffer_minutes, advance_booking_days_max,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (store_id) DO UPDATE SET
                requires_deposit          = EXCLUDED.requires_deposit,
                deposit_percentage        = EXCLUDED.deposit_percentage,
                cancellation_window_hours = EXCLUDED.cancellation_window_hours,
                no_show_fee_amount        = EXCLUDED.no_show_fee_amount,
                default_buffer_minutes    = EXCLUDED.default_buffer_minutes,
                advance_booking_days_max  = EXCLUDED.advance_booking_days_max,
                updated_at                = EXCLUDED.updated_at
            "#,
        )
        .bind(p.id().into_uuid())
        .bind(p.store_id())
        .bind(p.requires_deposit())
        .bind(p.deposit_percentage())
        .bind(p.cancellation_window_hours())
        .bind(p.no_show_fee_amount())
        .bind(p.default_buffer_minutes())
        .bind(p.advance_booking_days_max())
        .bind(p.created_at())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_store(&self, store_id: Uuid) -> Result<Option<BookingPolicy>, BookingError> {
        let row = sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT id, store_id, requires_deposit, deposit_percentage,
                   cancellation_window_hours, no_show_fee_amount,
                   default_buffer_minutes, advance_booking_days_max,
                   created_at, updated_at
            FROM booking_policies
            WHERE store_id = $1
            "#,
        )
        .bind(store_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(BookingPolicy::from))
    }
}

#[derive(sqlx::FromRow)]
struct PolicyRow {
    id: Uuid,
    store_id: Uuid,
    requires_deposit: bool,
    deposit_percentage: Option<Decimal>,
    cancellation_window_hours: i32,
    no_show_fee_amount: Option<Decimal>,
    default_buffer_minutes: i32,
    advance_booking_days_max: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PolicyRow> for BookingPolicy {
    fn from(r: PolicyRow) -> Self {
        BookingPolicy::reconstitute(
            BookingPolicyId::from_uuid(r.id),
            r.store_id,
            r.requires_deposit,
            r.deposit_percentage,
            r.cancellation_window_hours,
            r.no_show_fee_amount,
            r.default_buffer_minutes,
            r.advance_booking_days_max,
            r.created_at,
            r.updated_at,
        )
    }
}
