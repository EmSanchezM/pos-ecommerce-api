//! Pure forecasting math. No IO, no clock, no random — every function is a
//! deterministic transform from a series of `Decimal` to a forecast `Decimal`,
//! which makes them trivial to unit-test.
//!
//! v1 ships moving average and simple exponential smoothing. Holt-Winters is
//! provided as well but only used when at least `2 * season_length` points
//! exist; otherwise callers fall back to exponential smoothing.

mod exponential_smoothing;
mod holt_winters;
mod moving_average;
mod outliers;

pub use exponential_smoothing::exponential_smoothing;
pub use holt_winters::holt_winters_additive;
pub use moving_average::moving_average;
pub use outliers::filter_outliers;

use rust_decimal::Decimal;

/// Forecast result with a coarse confidence band — for v1 we use ±25% of the
/// point estimate, refined later with the residual standard deviation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForecastResult {
    pub point: Decimal,
    pub low: Decimal,
    pub high: Decimal,
}

impl ForecastResult {
    pub fn with_band(point: Decimal, band_factor: Decimal) -> Self {
        let half = point * band_factor;
        Self {
            point,
            low: (point - half).max(Decimal::ZERO),
            high: point + half,
        }
    }
}
