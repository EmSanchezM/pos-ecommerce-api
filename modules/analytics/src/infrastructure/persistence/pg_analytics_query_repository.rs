//! PostgreSQL implementation of AnalyticsQueryRepository.
//!
//! Cross-module aggregations against `sales`, `sale_items`, `products`,
//! `inventory_stock`, and `users`. We filter on `sales.status = 'completed'`
//! and use `completed_at` for time-window bounds — voided/draft sales are
//! intentionally excluded.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::AnalyticsError;
use crate::domain::entities::{
    CashierPerformanceRow, DeadStockRow, PeakHourRow, ProductProfitabilityRow,
};
use crate::domain::repositories::{AnalyticsQueryRepository, ReportFilters};

pub struct PgAnalyticsQueryRepository {
    pool: PgPool,
}

impl PgAnalyticsQueryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AnalyticsQueryRepository for PgAnalyticsQueryRepository {
    async fn total_revenue(
        &self,
        store_id: Option<Uuid>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Decimal, AnalyticsError> {
        let value: Option<Decimal> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(total), 0)::NUMERIC
            FROM sales
            WHERE status = 'completed'
              AND ($1::uuid IS NULL OR store_id = $1)
              AND completed_at >= $2
              AND completed_at < $3
            "#,
        )
        .bind(store_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await?;
        Ok(value.unwrap_or(Decimal::ZERO))
    }

    async fn sales_count(
        &self,
        store_id: Option<Uuid>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<i64, AnalyticsError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM sales
            WHERE status = 'completed'
              AND ($1::uuid IS NULL OR store_id = $1)
              AND completed_at >= $2
              AND completed_at < $3
            "#,
        )
        .bind(store_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn unique_customers(
        &self,
        store_id: Option<Uuid>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<i64, AnalyticsError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT customer_id)::BIGINT
            FROM sales
            WHERE status = 'completed'
              AND customer_id IS NOT NULL
              AND ($1::uuid IS NULL OR store_id = $1)
              AND completed_at >= $2
              AND completed_at < $3
            "#,
        )
        .bind(store_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn peak_hour(&self, f: &ReportFilters) -> Result<Vec<PeakHourRow>, AnalyticsError> {
        let rows = sqlx::query_as::<_, PeakHourRowDb>(
            r#"
            SELECT
                EXTRACT(DOW  FROM completed_at)::INT  AS day_of_week,
                EXTRACT(HOUR FROM completed_at)::INT  AS hour_of_day,
                COUNT(*)::BIGINT                       AS sales_count,
                COALESCE(SUM(total), 0)::NUMERIC       AS revenue
            FROM sales
            WHERE status = 'completed'
              AND ($1::uuid IS NULL OR store_id = $1)
              AND completed_at >= $2
              AND completed_at < $3
            GROUP BY day_of_week, hour_of_day
            ORDER BY day_of_week, hour_of_day
            LIMIT $4
            "#,
        )
        .bind(f.store_id)
        .bind(f.from)
        .bind(f.to)
        .bind(f.limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(PeakHourRow::from).collect())
    }

    async fn product_profitability(
        &self,
        f: &ReportFilters,
    ) -> Result<Vec<ProductProfitabilityRow>, AnalyticsError> {
        let rows = sqlx::query_as::<_, ProductProfitabilityRowDb>(
            r#"
            SELECT
                si.product_id,
                p.name                                                        AS product_name,
                COALESCE(SUM(si.quantity), 0)::NUMERIC                        AS units_sold,
                COALESCE(SUM(si.total), 0)::NUMERIC                           AS revenue,
                COALESCE(SUM(si.quantity * si.unit_cost), 0)::NUMERIC         AS estimated_cost,
                COALESCE(SUM(si.total) - SUM(si.quantity * si.unit_cost), 0)::NUMERIC
                                                                              AS gross_margin
            FROM sale_items si
            JOIN sales    s ON s.id = si.sale_id
            JOIN products p ON p.id = si.product_id
            WHERE s.status = 'completed'
              AND ($1::uuid IS NULL OR s.store_id = $1)
              AND s.completed_at >= $2
              AND s.completed_at < $3
            GROUP BY si.product_id, p.name
            ORDER BY revenue DESC
            LIMIT $4
            "#,
        )
        .bind(f.store_id)
        .bind(f.from)
        .bind(f.to)
        .bind(f.limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(ProductProfitabilityRow::from)
            .collect())
    }

    async fn dead_stock(&self, f: &ReportFilters) -> Result<Vec<DeadStockRow>, AnalyticsError> {
        let rows = sqlx::query_as::<_, DeadStockRowDb>(
            r#"
            SELECT
                inv.product_id                                                AS product_id,
                p.name                                                        AS product_name,
                MAX(s.completed_at)                                           AS last_sold_at,
                CASE
                    WHEN MAX(s.completed_at) IS NULL THEN NULL
                    ELSE EXTRACT(DAY FROM (NOW() - MAX(s.completed_at)))::INT
                END                                                           AS days_since_last_sale,
                inv.quantity                                                  AS current_stock_qty
            FROM inventory_stock inv
            JOIN products p ON p.id = inv.product_id
            LEFT JOIN sale_items si ON si.product_id = inv.product_id
            LEFT JOIN sales s
                ON s.id = si.sale_id
               AND s.status = 'completed'
               AND ($1::uuid IS NULL OR s.store_id = $1)
            WHERE inv.product_id IS NOT NULL
              AND ($1::uuid IS NULL OR inv.store_id = $1)
              AND inv.quantity > 0
            GROUP BY inv.product_id, p.name, inv.quantity
            HAVING MAX(s.completed_at) IS NULL OR MAX(s.completed_at) < $2
            ORDER BY days_since_last_sale DESC NULLS FIRST
            LIMIT $3
            "#,
        )
        .bind(f.store_id)
        .bind(f.from)
        .bind(f.limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(DeadStockRow::from).collect())
    }

    async fn cashier_performance(
        &self,
        f: &ReportFilters,
    ) -> Result<Vec<CashierPerformanceRow>, AnalyticsError> {
        let rows = sqlx::query_as::<_, CashierPerformanceRowDb>(
            r#"
            SELECT
                s.cashier_id                                                  AS user_id,
                (u.first_name || ' ' || u.last_name)                          AS user_name,
                COUNT(*)::BIGINT                                              AS sales_count,
                COALESCE(SUM(s.total), 0)::NUMERIC                            AS revenue,
                COALESCE(AVG(s.total), 0)::NUMERIC                            AS average_ticket
            FROM sales s
            JOIN users u ON u.id = s.cashier_id
            WHERE s.status = 'completed'
              AND s.cashier_id IS NOT NULL
              AND ($1::uuid IS NULL OR s.store_id = $1)
              AND s.completed_at >= $2
              AND s.completed_at < $3
            GROUP BY s.cashier_id, u.first_name, u.last_name
            ORDER BY revenue DESC
            LIMIT $4
            "#,
        )
        .bind(f.store_id)
        .bind(f.from)
        .bind(f.to)
        .bind(f.limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(CashierPerformanceRow::from).collect())
    }
}

// -----------------------------------------------------------------------------
// Row adapters — sqlx requires concrete types for FromRow.
// -----------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct PeakHourRowDb {
    day_of_week: i32,
    hour_of_day: i32,
    sales_count: i64,
    revenue: Decimal,
}

impl From<PeakHourRowDb> for PeakHourRow {
    fn from(r: PeakHourRowDb) -> Self {
        Self {
            day_of_week: r.day_of_week,
            hour_of_day: r.hour_of_day,
            sales_count: r.sales_count,
            revenue: r.revenue,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ProductProfitabilityRowDb {
    product_id: Uuid,
    product_name: String,
    units_sold: Decimal,
    revenue: Decimal,
    estimated_cost: Decimal,
    gross_margin: Decimal,
}

impl From<ProductProfitabilityRowDb> for ProductProfitabilityRow {
    fn from(r: ProductProfitabilityRowDb) -> Self {
        Self {
            product_id: r.product_id,
            product_name: r.product_name,
            units_sold: r.units_sold,
            revenue: r.revenue,
            estimated_cost: r.estimated_cost,
            gross_margin: r.gross_margin,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DeadStockRowDb {
    product_id: Uuid,
    product_name: String,
    last_sold_at: Option<DateTime<Utc>>,
    days_since_last_sale: Option<i32>,
    current_stock_qty: Decimal,
}

impl From<DeadStockRowDb> for DeadStockRow {
    fn from(r: DeadStockRowDb) -> Self {
        Self {
            product_id: r.product_id,
            product_name: r.product_name,
            last_sold_at: r.last_sold_at,
            days_since_last_sale: r.days_since_last_sale,
            current_stock_qty: r.current_stock_qty,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CashierPerformanceRowDb {
    user_id: Uuid,
    user_name: String,
    sales_count: i64,
    revenue: Decimal,
    average_ticket: Decimal,
}

impl From<CashierPerformanceRowDb> for CashierPerformanceRow {
    fn from(r: CashierPerformanceRowDb) -> Self {
        Self {
            user_id: r.user_id,
            user_name: r.user_name,
            sales_count: r.sales_count,
            revenue: r.revenue,
            average_ticket: r.average_ticket,
        }
    }
}
