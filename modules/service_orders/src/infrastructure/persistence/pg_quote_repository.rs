use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::domain::entities::Quote;
use crate::domain::repositories::QuoteRepository;
use crate::domain::value_objects::{QuoteId, QuoteStatus, ServiceOrderId};

pub struct PgQuoteRepository {
    pool: PgPool,
}

impl PgQuoteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QuoteRepository for PgQuoteRepository {
    async fn save(&self, q: &Quote) -> Result<(), ServiceOrdersError> {
        sqlx::query(
            r#"
            INSERT INTO service_quotes (
                id, service_order_id, version,
                labor_total, parts_total, tax_total, grand_total,
                valid_until, notes, status, sent_at, decided_at,
                decided_by_customer, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(q.id().into_uuid())
        .bind(q.service_order_id().into_uuid())
        .bind(q.version())
        .bind(q.labor_total())
        .bind(q.parts_total())
        .bind(q.tax_total())
        .bind(q.grand_total())
        .bind(q.valid_until())
        .bind(q.notes())
        .bind(q.status().as_str())
        .bind(q.sent_at())
        .bind(q.decided_at())
        .bind(q.decided_by_customer())
        .bind(q.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, q: &Quote) -> Result<(), ServiceOrdersError> {
        let result = sqlx::query(
            r#"
            UPDATE service_quotes
               SET status              = $2,
                   sent_at             = $3,
                   decided_at          = $4,
                   decided_by_customer = $5,
                   notes               = $6,
                   valid_until         = $7
             WHERE id = $1
            "#,
        )
        .bind(q.id().into_uuid())
        .bind(q.status().as_str())
        .bind(q.sent_at())
        .bind(q.decided_at())
        .bind(q.decided_by_customer())
        .bind(q.notes())
        .bind(q.valid_until())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ServiceOrdersError::QuoteNotFound(q.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(&self, id: QuoteId) -> Result<Option<Quote>, ServiceOrdersError> {
        let row = sqlx::query_as::<_, QuoteRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(Quote::try_from).transpose()
    }

    async fn list_by_order(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Vec<Quote>, ServiceOrdersError> {
        let rows = sqlx::query_as::<_, QuoteRow>(LIST_BY_ORDER)
            .bind(order_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Quote::try_from).collect()
    }

    async fn max_version_for_order(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<i32, ServiceOrdersError> {
        let row: (Option<i32>,) =
            sqlx::query_as("SELECT MAX(version) FROM service_quotes WHERE service_order_id = $1")
                .bind(order_id.into_uuid())
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0.unwrap_or(0))
    }

    async fn mark_others_superseded(
        &self,
        order_id: ServiceOrderId,
        except_id: QuoteId,
    ) -> Result<(), ServiceOrdersError> {
        sqlx::query(
            r#"
            UPDATE service_quotes
               SET status = 'superseded'
             WHERE service_order_id = $1
               AND id <> $2
               AND status IN ('draft', 'sent')
            "#,
        )
        .bind(order_id.into_uuid())
        .bind(except_id.into_uuid())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, service_order_id, version,
       labor_total, parts_total, tax_total, grand_total,
       valid_until, notes, status, sent_at, decided_at,
       decided_by_customer, created_at
FROM service_quotes
WHERE id = $1
"#;

const LIST_BY_ORDER: &str = r#"
SELECT id, service_order_id, version,
       labor_total, parts_total, tax_total, grand_total,
       valid_until, notes, status, sent_at, decided_at,
       decided_by_customer, created_at
FROM service_quotes
WHERE service_order_id = $1
ORDER BY version DESC
"#;

#[derive(sqlx::FromRow)]
struct QuoteRow {
    id: Uuid,
    service_order_id: Uuid,
    version: i32,
    labor_total: Decimal,
    parts_total: Decimal,
    tax_total: Decimal,
    grand_total: Decimal,
    valid_until: Option<DateTime<Utc>>,
    notes: Option<String>,
    status: String,
    sent_at: Option<DateTime<Utc>>,
    decided_at: Option<DateTime<Utc>>,
    decided_by_customer: bool,
    created_at: DateTime<Utc>,
}

impl TryFrom<QuoteRow> for Quote {
    type Error = ServiceOrdersError;
    fn try_from(r: QuoteRow) -> Result<Self, ServiceOrdersError> {
        Ok(Quote::reconstitute(
            QuoteId::from_uuid(r.id),
            ServiceOrderId::from_uuid(r.service_order_id),
            r.version,
            r.labor_total,
            r.parts_total,
            r.tax_total,
            r.grand_total,
            r.valid_until,
            r.notes,
            QuoteStatus::from_str(&r.status)?,
            r.sent_at,
            r.decided_at,
            r.decided_by_customer,
            r.created_at,
        ))
    }
}
