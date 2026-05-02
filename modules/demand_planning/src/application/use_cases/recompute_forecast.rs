//! RecomputeForecastUseCase — for every (variant, store) pair with sales in
//! the lookback window, fetches the daily series, runs each forecasting method
//! we ship in v1, and persists one row per method.
//!
//! Designed to be invoked by the nightly job. Returns the number of forecast
//! rows written so the caller can log it.

use std::sync::Arc;

use chrono::{Duration, Utc};
use rust_decimal::Decimal;

use crate::DemandPlanningError;
use crate::application::forecasting::{
    ForecastResult, exponential_smoothing, filter_outliers, holt_winters_additive, moving_average,
};
use crate::domain::repositories::{DemandForecastRepository, SalesHistoryRepository};
use crate::domain::value_objects::{ForecastMethod, ForecastPeriod};

/// How many days of history to consider on each pass. 180 days covers two
/// quarters which is enough for monthly seasonality on most retail SKUs.
const LOOKBACK_DAYS: i64 = 180;

/// Confidence band as a fraction of the point estimate. v1 uses ±25%.
const CONFIDENCE_BAND_NUM: i64 = 25;
const CONFIDENCE_BAND_DEN: i64 = 100;

pub struct RecomputeForecastUseCase {
    history: Arc<dyn SalesHistoryRepository>,
    forecasts: Arc<dyn DemandForecastRepository>,
}

impl RecomputeForecastUseCase {
    pub fn new(
        history: Arc<dyn SalesHistoryRepository>,
        forecasts: Arc<dyn DemandForecastRepository>,
    ) -> Self {
        Self { history, forecasts }
    }

    pub async fn execute(&self) -> Result<usize, DemandPlanningError> {
        let today = Utc::now().date_naive();
        let from = today - Duration::days(LOOKBACK_DAYS);

        let pairs = self.history.active_variants(from, today).await?;
        let mut written = 0usize;
        let band = Decimal::new(CONFIDENCE_BAND_NUM, 0) / Decimal::new(CONFIDENCE_BAND_DEN, 0);

        for (variant_id, store_id) in pairs {
            let raw = self
                .history
                .aggregate_units_sold(variant_id, store_id, from, today, ForecastPeriod::Daily)
                .await?;
            if raw.is_empty() {
                continue;
            }
            let cleaned: Vec<Decimal> =
                filter_outliers(&raw.iter().map(|p| p.units).collect::<Vec<_>>(), 3.0);
            if cleaned.is_empty() {
                continue;
            }

            let next_start = today + Duration::days(1);
            let next_end = next_start + Duration::days(6); // forecast horizon: next week

            // We forecast a daily rate, then scale to the horizon (7 days).
            let horizon = Decimal::from(7u32);

            // Moving average (3 / 6 windows). The series is daily; window 3
            // and 6 correspond to "last 3 / 6 days" — coarse but useful.
            let ma3 = moving_average(&cleaned, 3) * horizon;
            let ma6 = moving_average(&cleaned, 6) * horizon;
            // Simple exponential smoothing with alpha = 0.3.
            let ses = exponential_smoothing(&cleaned, 0.3) * horizon;
            // Holt-Winters with weekly seasonality (m = 7) when we have enough.
            let hw = holt_winters_additive(&cleaned, 0.3, 0.1, 0.3, 7);

            let methods: Vec<(ForecastMethod, Decimal)> = match hw {
                Ok(value) => vec![
                    (ForecastMethod::MovingAverage3, ma3),
                    (ForecastMethod::MovingAverage6, ma6),
                    (ForecastMethod::ExponentialSmoothing, ses),
                    (ForecastMethod::HoltWinters, value * horizon),
                ],
                Err(_) => vec![
                    (ForecastMethod::MovingAverage3, ma3),
                    (ForecastMethod::MovingAverage6, ma6),
                    (ForecastMethod::ExponentialSmoothing, ses),
                ],
            };

            for (method, point) in methods {
                let result = ForecastResult::with_band(point.max(Decimal::ZERO), band);
                self.forecasts
                    .record(
                        variant_id,
                        store_id,
                        ForecastPeriod::Weekly,
                        next_start,
                        next_end,
                        method,
                        result.point,
                        result.low,
                        result.high,
                    )
                    .await?;
                written += 1;
            }
        }

        Ok(written)
    }

    /// Convenience constant exported so callers (the recompute job, tests) can
    /// reason about the lookback window without touching this file.
    pub fn lookback_days() -> i64 {
        LOOKBACK_DAYS
    }
}
