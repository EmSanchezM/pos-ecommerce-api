//! DemandForecast entity — the statistical forecast of how many units of a
//! product variant a store will sell in the next period. Persisted only after
//! a successful recompute; older rows are pruned by the recompute job.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{ForecastId, ForecastMethod, ForecastPeriod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandForecast {
    id: ForecastId,
    product_variant_id: Uuid,
    store_id: Uuid,
    period: ForecastPeriod,
    period_start: NaiveDate,
    period_end: NaiveDate,
    method: ForecastMethod,
    forecasted_qty: Decimal,
    confidence_low: Decimal,
    confidence_high: Decimal,
    computed_at: DateTime<Utc>,
}

impl DemandForecast {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        product_variant_id: Uuid,
        store_id: Uuid,
        period: ForecastPeriod,
        period_start: NaiveDate,
        period_end: NaiveDate,
        method: ForecastMethod,
        forecasted_qty: Decimal,
        confidence_low: Decimal,
        confidence_high: Decimal,
    ) -> Self {
        Self {
            id: ForecastId::new(),
            product_variant_id,
            store_id,
            period,
            period_start,
            period_end,
            method,
            forecasted_qty,
            confidence_low,
            confidence_high,
            computed_at: Utc::now(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ForecastId,
        product_variant_id: Uuid,
        store_id: Uuid,
        period: ForecastPeriod,
        period_start: NaiveDate,
        period_end: NaiveDate,
        method: ForecastMethod,
        forecasted_qty: Decimal,
        confidence_low: Decimal,
        confidence_high: Decimal,
        computed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            product_variant_id,
            store_id,
            period,
            period_start,
            period_end,
            method,
            forecasted_qty,
            confidence_low,
            confidence_high,
            computed_at,
        }
    }

    pub fn id(&self) -> ForecastId {
        self.id
    }
    pub fn product_variant_id(&self) -> Uuid {
        self.product_variant_id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn period(&self) -> ForecastPeriod {
        self.period
    }
    pub fn period_start(&self) -> NaiveDate {
        self.period_start
    }
    pub fn period_end(&self) -> NaiveDate {
        self.period_end
    }
    pub fn method(&self) -> ForecastMethod {
        self.method
    }
    pub fn forecasted_qty(&self) -> Decimal {
        self.forecasted_qty
    }
    pub fn confidence_low(&self) -> Decimal {
        self.confidence_low
    }
    pub fn confidence_high(&self) -> Decimal {
        self.confidence_high
    }
    pub fn computed_at(&self) -> DateTime<Utc> {
        self.computed_at
    }
}
