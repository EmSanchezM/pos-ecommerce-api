//! Read-only projection over the existing `sales` / `sale_items` tables. The
//! demand_planning module never writes here; it only aggregates.

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::SeriesPoint;
use crate::domain::value_objects::ForecastPeriod;

#[async_trait]
pub trait SalesHistoryRepository: Send + Sync {
    /// Return a chronologically ordered series of (period_start, units sold)
    /// for a (variant, store) tuple.
    async fn aggregate_units_sold(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        granularity: ForecastPeriod,
    ) -> Result<Vec<SeriesPoint>, DemandPlanningError>;

    /// All (variant, store) pairs that have at least one completed sale in the
    /// window. Drives the recompute job's outer loop.
    async fn active_variants(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<(Uuid, Uuid)>, DemandPlanningError>;

    /// Revenue per variant in a window — used by ABC classification.
    async fn revenue_by_variant(
        &self,
        store_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<RevenueRow>, DemandPlanningError>;
}

#[derive(Debug, Clone)]
pub struct RevenueRow {
    pub product_variant_id: Uuid,
    pub store_id: Uuid,
    pub revenue: Decimal,
}
