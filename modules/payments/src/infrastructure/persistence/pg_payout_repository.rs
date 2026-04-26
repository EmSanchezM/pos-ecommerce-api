//! PostgreSQL implementation of PayoutRepository.

use async_trait::async_trait;
use sqlx::PgPool;

use crate::PaymentsError;
use crate::domain::entities::Payout;
use crate::domain::repositories::PayoutRepository;
use crate::domain::value_objects::{PaymentGatewayId, PayoutId, PayoutStatus};
use identity::StoreId;

pub struct PgPayoutRepository {
    pool: PgPool,
}

impl PgPayoutRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PayoutRepository for PgPayoutRepository {
    async fn save(&self, payout: &Payout) -> Result<(), PaymentsError> {
        sqlx::query(
            r#"
            INSERT INTO payouts (
                id, store_id, gateway_id, status, amount, currency,
                fee_amount, net_amount, gateway_payout_id, transaction_count,
                period_start, period_end, expected_arrival, arrived_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(payout.id().into_uuid())
        .bind(payout.store_id().into_uuid())
        .bind(payout.gateway_id().into_uuid())
        .bind(payout.status().to_string())
        .bind(payout.amount())
        .bind(payout.currency())
        .bind(payout.fee_amount())
        .bind(payout.net_amount())
        .bind(payout.gateway_payout_id())
        .bind(payout.transaction_count())
        .bind(payout.period_start())
        .bind(payout.period_end())
        .bind(payout.expected_arrival())
        .bind(payout.arrived_at())
        .bind(payout.created_at())
        .bind(payout.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: PayoutId) -> Result<Option<Payout>, PaymentsError> {
        let row = sqlx::query_as::<_, PayoutRow>(SELECT_PAYOUT_BY_ID_SQL)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_store(
        &self,
        store_id: StoreId,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Payout>, i64), PaymentsError> {
        let offset = (page - 1) * page_size;
        let rows = sqlx::query_as::<_, PayoutRow>(
            r#"
            SELECT id, store_id, gateway_id, status, amount, currency,
                   fee_amount, net_amount, gateway_payout_id, transaction_count,
                   period_start, period_end, expected_arrival, arrived_at,
                   created_at, updated_at
            FROM payouts
            WHERE store_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM payouts WHERE store_id = $1")
            .bind(store_id.into_uuid())
            .fetch_one(&self.pool)
            .await?;

        let items: Result<Vec<Payout>, _> = rows.into_iter().map(|r| r.try_into()).collect();
        Ok((items?, total.0))
    }

    async fn update(&self, payout: &Payout) -> Result<(), PaymentsError> {
        let result = sqlx::query(
            r#"
            UPDATE payouts
            SET status = $2,
                amount = $3,
                fee_amount = $4,
                net_amount = $5,
                gateway_payout_id = $6,
                transaction_count = $7,
                expected_arrival = $8,
                arrived_at = $9,
                updated_at = $10
            WHERE id = $1
            "#,
        )
        .bind(payout.id().into_uuid())
        .bind(payout.status().to_string())
        .bind(payout.amount())
        .bind(payout.fee_amount())
        .bind(payout.net_amount())
        .bind(payout.gateway_payout_id())
        .bind(payout.transaction_count())
        .bind(payout.expected_arrival())
        .bind(payout.arrived_at())
        .bind(payout.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PaymentsError::PayoutNotFound(payout.id().into_uuid()));
        }
        Ok(())
    }
}

const SELECT_PAYOUT_BY_ID_SQL: &str = r#"
SELECT id, store_id, gateway_id, status, amount, currency,
       fee_amount, net_amount, gateway_payout_id, transaction_count,
       period_start, period_end, expected_arrival, arrived_at,
       created_at, updated_at
FROM payouts
WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct PayoutRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    gateway_id: uuid::Uuid,
    status: String,
    amount: rust_decimal::Decimal,
    currency: String,
    fee_amount: rust_decimal::Decimal,
    net_amount: rust_decimal::Decimal,
    gateway_payout_id: Option<String>,
    transaction_count: i32,
    period_start: chrono::DateTime<chrono::Utc>,
    period_end: chrono::DateTime<chrono::Utc>,
    expected_arrival: Option<chrono::DateTime<chrono::Utc>>,
    arrived_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<PayoutRow> for Payout {
    type Error = PaymentsError;

    fn try_from(row: PayoutRow) -> Result<Self, Self::Error> {
        let status: PayoutStatus = row
            .status
            .parse()
            .map_err(|_| PaymentsError::InvalidPayoutStatus)?;
        Ok(Payout::reconstitute(
            PayoutId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            PaymentGatewayId::from_uuid(row.gateway_id),
            status,
            row.amount,
            row.currency,
            row.fee_amount,
            row.net_amount,
            row.gateway_payout_id,
            row.transaction_count,
            row.period_start,
            row.period_end,
            row.expected_arrival,
            row.arrived_at,
            row.created_at,
            row.updated_at,
        ))
    }
}
