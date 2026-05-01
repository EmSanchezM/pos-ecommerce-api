//! AnalyticsQueryRepository — cross-module aggregations.
//!
//! Implementations run raw SQL against the existing tables of `sales`,
//! `inventory`, `purchasing`, etc. without depending on those crates as Rust
//! code, so analytics stays decoupled from internal refactors there.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::AnalyticsError;
use crate::domain::entities::{
    CashierPerformanceRow, DeadStockRow, PeakHourRow, ProductProfitabilityRow,
};

/// Filters applied to all reports.
#[derive(Debug, Clone)]
pub struct ReportFilters {
    pub store_id: Option<Uuid>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub limit: i64,
}

#[async_trait]
pub trait AnalyticsQueryRepository: Send + Sync {
    // ---- Aggregates used by RecomputeKpiSnapshotsUseCase --------------------

    async fn total_revenue(
        &self,
        store_id: Option<Uuid>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Decimal, AnalyticsError>;

    async fn sales_count(
        &self,
        store_id: Option<Uuid>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<i64, AnalyticsError>;

    async fn unique_customers(
        &self,
        store_id: Option<Uuid>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<i64, AnalyticsError>;

    // ---- Reports ------------------------------------------------------------

    async fn peak_hour(&self, filters: &ReportFilters) -> Result<Vec<PeakHourRow>, AnalyticsError>;

    async fn product_profitability(
        &self,
        filters: &ReportFilters,
    ) -> Result<Vec<ProductProfitabilityRow>, AnalyticsError>;

    async fn dead_stock(
        &self,
        filters: &ReportFilters,
    ) -> Result<Vec<DeadStockRow>, AnalyticsError>;

    async fn cashier_performance(
        &self,
        filters: &ReportFilters,
    ) -> Result<Vec<CashierPerformanceRow>, AnalyticsError>;
}
