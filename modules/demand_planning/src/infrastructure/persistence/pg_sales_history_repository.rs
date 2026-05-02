//! PostgreSQL projection over `sales` + `sale_items`. Aggregates units sold by
//! day/week/month, returns the (variant, store) pairs that have completed
//! sales in a window, and totals revenue for ABC classification.
//!
//! Variants without an explicit `variant_id` (i.e. the simple-product case)
//! still need replenishment, so the queries fall back to `product_id` cast as
//! a "virtual variant id" — the column convention everywhere else in the API
//! treats `variant_id IS NULL` as "the product itself". This module follows
//! the same convention so its outputs line up with reorder policies and stock.

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::SeriesPoint;
use crate::domain::repositories::{RevenueRow, SalesHistoryRepository};
use crate::domain::value_objects::ForecastPeriod;

pub struct PgSalesHistoryRepository {
    pool: PgPool,
}

impl PgSalesHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SalesHistoryRepository for PgSalesHistoryRepository {
    async fn aggregate_units_sold(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        granularity: ForecastPeriod,
    ) -> Result<Vec<SeriesPoint>, DemandPlanningError> {
        let trunc = match granularity {
            ForecastPeriod::Daily => "day",
            ForecastPeriod::Weekly => "week",
            ForecastPeriod::Monthly => "month",
        };
        let sql = format!(
            r#"
            SELECT date_trunc('{trunc}', s.completed_at)::DATE AS bucket,
                   COALESCE(SUM(si.quantity), 0)::NUMERIC AS units
            FROM sales s
            JOIN sale_items si ON si.sale_id = s.id
            WHERE s.store_id = $1
              AND s.status = 'completed'
              AND s.completed_at IS NOT NULL
              AND s.completed_at >= $2
              AND s.completed_at < $3
              AND COALESCE(si.variant_id, si.product_id) = $4
            GROUP BY bucket
            ORDER BY bucket ASC
            "#,
        );
        let rows: Vec<(NaiveDate, Decimal)> = sqlx::query_as(&sql)
            .bind(store_id)
            .bind(from)
            .bind(to)
            .bind(product_variant_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|(d, u)| SeriesPoint::new(d, u))
            .collect())
    }

    async fn active_variants(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<(Uuid, Uuid)>, DemandPlanningError> {
        let rows: Vec<(Uuid, Uuid)> = sqlx::query_as(
            r#"
            SELECT DISTINCT COALESCE(si.variant_id, si.product_id) AS variant_id, s.store_id
            FROM sales s
            JOIN sale_items si ON si.sale_id = s.id
            WHERE s.status = 'completed'
              AND s.completed_at IS NOT NULL
              AND s.completed_at >= $1
              AND s.completed_at < $2
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn revenue_by_variant(
        &self,
        store_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<RevenueRow>, DemandPlanningError> {
        let rows: Vec<(Uuid, Uuid, Decimal)> = match store_id {
            Some(s) => {
                sqlx::query_as(
                    r#"
                    SELECT COALESCE(si.variant_id, si.product_id) AS variant_id,
                           s.store_id,
                           COALESCE(SUM(si.total), 0)::NUMERIC AS revenue
                    FROM sales s
                    JOIN sale_items si ON si.sale_id = s.id
                    WHERE s.status = 'completed'
                      AND s.completed_at IS NOT NULL
                      AND s.completed_at >= $1
                      AND s.completed_at < $2
                      AND s.store_id = $3
                    GROUP BY COALESCE(si.variant_id, si.product_id), s.store_id
                    "#,
                )
                .bind(from)
                .bind(to)
                .bind(s)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as(
                    r#"
                    SELECT COALESCE(si.variant_id, si.product_id) AS variant_id,
                           s.store_id,
                           COALESCE(SUM(si.total), 0)::NUMERIC AS revenue
                    FROM sales s
                    JOIN sale_items si ON si.sale_id = s.id
                    WHERE s.status = 'completed'
                      AND s.completed_at IS NOT NULL
                      AND s.completed_at >= $1
                      AND s.completed_at < $2
                    GROUP BY COALESCE(si.variant_id, si.product_id), s.store_id
                    "#,
                )
                .bind(from)
                .bind(to)
                .fetch_all(&self.pool)
                .await?
            }
        };
        Ok(rows
            .into_iter()
            .map(|(variant_id, store_id, revenue)| RevenueRow {
                product_variant_id: variant_id,
                store_id,
                revenue,
            })
            .collect())
    }
}
