use async_trait::async_trait;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::DemandForecast;
use crate::domain::value_objects::{ForecastMethod, ForecastPeriod};

#[async_trait]
pub trait DemandForecastRepository: Send + Sync {
    /// Persist a freshly computed forecast row.
    async fn save(&self, forecast: &DemandForecast) -> Result<(), DemandPlanningError>;

    /// Return the most recent forecast for a (variant, store) tuple regardless of method.
    async fn find_latest(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Option<DemandForecast>, DemandPlanningError>;

    /// Return the latest forecast that matches a specific method.
    async fn find_latest_by_method(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
        method: ForecastMethod,
    ) -> Result<Option<DemandForecast>, DemandPlanningError>;

    /// Return all forecasts for a (variant, store) tuple, newest first.
    async fn list_for_variant(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Vec<DemandForecast>, DemandPlanningError>;

    /// Delete forecasts older than the cutoff so the table doesn't grow forever.
    async fn delete_older_than(
        &self,
        cutoff: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, DemandPlanningError>;

    /// Convenience constructor that records a freshly built forecast row using
    /// the provided period/method classifiers (used by the recompute job).
    #[allow(clippy::too_many_arguments)]
    async fn record(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
        period: ForecastPeriod,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        method: ForecastMethod,
        forecasted_qty: rust_decimal::Decimal,
        confidence_low: rust_decimal::Decimal,
        confidence_high: rust_decimal::Decimal,
    ) -> Result<DemandForecast, DemandPlanningError> {
        let forecast = DemandForecast::create(
            product_variant_id,
            store_id,
            period,
            period_start,
            period_end,
            method,
            forecasted_qty,
            confidence_low,
            confidence_high,
        );
        self.save(&forecast).await?;
        Ok(forecast)
    }
}
