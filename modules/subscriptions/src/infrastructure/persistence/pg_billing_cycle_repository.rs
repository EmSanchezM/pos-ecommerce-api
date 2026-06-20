use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::SubscriptionError;
use crate::domain::entities::BillingCycle;
use crate::domain::repositories::BillingCycleRepository;
use crate::domain::value_objects::{BillingCycleId, BillingCycleStatus, SubscriptionId};

pub struct PgBillingCycleRepository {
    pool: PgPool,
}

impl PgBillingCycleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BillingCycleRepository for PgBillingCycleRepository {
    async fn save(&self, c: &BillingCycle) -> Result<(), SubscriptionError> {
        sqlx::query(
            r#"
            INSERT INTO billing_cycles (
                id, subscription_id, period_start, period_end, status,
                invoice_id, transaction_id, amount_cents, currency,
                attempted_at, settled_at, failure_reason, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(c.id().into_uuid())
        .bind(c.subscription_id().into_uuid())
        .bind(c.period_start())
        .bind(c.period_end())
        .bind(c.status().as_str())
        .bind(c.invoice_id())
        .bind(c.transaction_id())
        .bind(c.amount_cents())
        .bind(c.currency())
        .bind(c.attempted_at())
        .bind(c.settled_at())
        .bind(c.failure_reason())
        .bind(c.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, c: &BillingCycle) -> Result<(), SubscriptionError> {
        let result = sqlx::query(
            r#"
            UPDATE billing_cycles
               SET status         = $2,
                   invoice_id     = $3,
                   transaction_id = $4,
                   amount_cents   = $5,
                   attempted_at   = $6,
                   settled_at     = $7,
                   failure_reason = $8
             WHERE id = $1
            "#,
        )
        .bind(c.id().into_uuid())
        .bind(c.status().as_str())
        .bind(c.invoice_id())
        .bind(c.transaction_id())
        .bind(c.amount_cents())
        .bind(c.attempted_at())
        .bind(c.settled_at())
        .bind(c.failure_reason())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(SubscriptionError::BillingCycleNotFound(c.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: BillingCycleId,
    ) -> Result<Option<BillingCycle>, SubscriptionError> {
        find_by_id_q(&self.pool, id).await
    }

    async fn find_by_id_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: BillingCycleId,
    ) -> Result<Option<BillingCycle>, SubscriptionError> {
        find_by_id_q(&mut **tx, id).await
    }

    async fn find_by_transaction_id(
        &self,
        transaction_id: Uuid,
    ) -> Result<Option<BillingCycle>, SubscriptionError> {
        let row = sqlx::query_as::<_, CycleRow>(SELECT_BY_TRANSACTION)
            .bind(transaction_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(BillingCycle::try_from).transpose()
    }

    async fn find_by_subscription(
        &self,
        subscription_id: SubscriptionId,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<BillingCycle>, i64), SubscriptionError> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 200);
        let offset = (page - 1) * page_size;

        let (total,): (i64,) =
            sqlx::query_as(r#"SELECT COUNT(*) FROM billing_cycles WHERE subscription_id = $1"#)
                .bind(subscription_id.into_uuid())
                .fetch_one(&self.pool)
                .await?;

        let rows = sqlx::query_as::<_, CycleRow>(SELECT_BY_SUBSCRIPTION)
            .bind(subscription_id.into_uuid())
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
        let cycles: Result<Vec<BillingCycle>, SubscriptionError> =
            rows.into_iter().map(BillingCycle::try_from).collect();
        Ok((cycles?, total))
    }

    async fn find_pending_due(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<BillingCycle>, SubscriptionError> {
        let rows = sqlx::query_as::<_, CycleRow>(SELECT_PENDING_DUE)
            .bind(now)
            .bind(limit.max(1))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(BillingCycle::try_from).collect()
    }
}

async fn find_by_id_q<'e, E: PgExecutor<'e>>(
    exec: E,
    id: BillingCycleId,
) -> Result<Option<BillingCycle>, SubscriptionError> {
    let row = sqlx::query_as::<_, CycleRow>(SELECT_BY_ID)
        .bind(id.into_uuid())
        .fetch_optional(exec)
        .await?;
    row.map(BillingCycle::try_from).transpose()
}

const SELECT_BY_ID: &str = r#"
SELECT id, subscription_id, period_start, period_end, status,
       invoice_id, transaction_id, amount_cents, currency,
       attempted_at, settled_at, failure_reason, created_at
FROM billing_cycles
WHERE id = $1
"#;

const SELECT_BY_TRANSACTION: &str = r#"
SELECT id, subscription_id, period_start, period_end, status,
       invoice_id, transaction_id, amount_cents, currency,
       attempted_at, settled_at, failure_reason, created_at
FROM billing_cycles
WHERE transaction_id = $1
"#;

const SELECT_BY_SUBSCRIPTION: &str = r#"
SELECT id, subscription_id, period_start, period_end, status,
       invoice_id, transaction_id, amount_cents, currency,
       attempted_at, settled_at, failure_reason, created_at
FROM billing_cycles
WHERE subscription_id = $1
ORDER BY period_end DESC
LIMIT $2 OFFSET $3
"#;

const SELECT_PENDING_DUE: &str = r#"
SELECT id, subscription_id, period_start, period_end, status,
       invoice_id, transaction_id, amount_cents, currency,
       attempted_at, settled_at, failure_reason, created_at
FROM billing_cycles
WHERE status = 'pending' AND period_start <= $1
ORDER BY period_start ASC
LIMIT $2
"#;

#[derive(sqlx::FromRow)]
struct CycleRow {
    id: Uuid,
    subscription_id: Uuid,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    status: String,
    invoice_id: Option<Uuid>,
    transaction_id: Option<Uuid>,
    amount_cents: i64,
    currency: String,
    attempted_at: Option<DateTime<Utc>>,
    settled_at: Option<DateTime<Utc>>,
    failure_reason: Option<String>,
    created_at: DateTime<Utc>,
}

impl TryFrom<CycleRow> for BillingCycle {
    type Error = SubscriptionError;
    fn try_from(r: CycleRow) -> Result<Self, SubscriptionError> {
        Ok(BillingCycle::reconstitute(
            BillingCycleId::from_uuid(r.id),
            SubscriptionId::from_uuid(r.subscription_id),
            r.period_start,
            r.period_end,
            BillingCycleStatus::from_str(&r.status)?,
            r.invoice_id,
            r.transaction_id,
            r.amount_cents,
            r.currency,
            r.attempted_at,
            r.settled_at,
            r.failure_reason,
            r.created_at,
        ))
    }
}
