//! Holt-Winters additive seasonal smoothing — the textbook formulation:
//!
//! ```text
//! L_t = alpha * (x_t - S_{t-m}) + (1-alpha)*(L_{t-1} + T_{t-1})
//! T_t = beta  * (L_t - L_{t-1}) + (1-beta) * T_{t-1}
//! S_t = gamma * (x_t - L_t)     + (1-gamma)* S_{t-m}
//! F_{t+1} = L_t + T_t + S_{t+1-m}
//! ```
//!
//! Requires at least `2 * season_length` data points; otherwise returns
//! `InsufficientHistory` so callers can fall back to a simpler method.

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use crate::DemandPlanningError;

#[allow(clippy::needless_range_loop)]
pub fn holt_winters_additive(
    series: &[Decimal],
    alpha: f64,
    beta: f64,
    gamma: f64,
    season_length: usize,
) -> Result<Decimal, DemandPlanningError> {
    let needed = season_length * 2;
    if series.len() < needed {
        return Err(DemandPlanningError::InsufficientHistory {
            needed,
            got: series.len(),
        });
    }

    let alpha = alpha.clamp(0.0, 1.0);
    let beta = beta.clamp(0.0, 1.0);
    let gamma = gamma.clamp(0.0, 1.0);

    let alpha_d = Decimal::from_f64(alpha).unwrap_or(Decimal::ZERO);
    let beta_d = Decimal::from_f64(beta).unwrap_or(Decimal::ZERO);
    let gamma_d = Decimal::from_f64(gamma).unwrap_or(Decimal::ZERO);

    // Initialise level from the first season's mean and trend from the slope
    // between the first two seasons.
    let first_season: Decimal = series[..season_length].iter().copied().sum::<Decimal>()
        / Decimal::from(season_length as u64);
    let second_season: Decimal = series[season_length..2 * season_length]
        .iter()
        .copied()
        .sum::<Decimal>()
        / Decimal::from(season_length as u64);
    let mut level = first_season;
    let mut trend = (second_season - first_season) / Decimal::from(season_length as u64);

    // Initial seasonal indices: x_i - mean of first season for the first
    // season, repeated.
    let mut seasonal: Vec<Decimal> = (0..season_length)
        .map(|i| series[i] - first_season)
        .collect();

    for t in 0..series.len() {
        let prev_level = level;
        let s_index = t % season_length;
        let s_prev = seasonal[s_index];
        level = alpha_d * (series[t] - s_prev) + (Decimal::ONE - alpha_d) * (prev_level + trend);
        trend = beta_d * (level - prev_level) + (Decimal::ONE - beta_d) * trend;
        seasonal[s_index] = gamma_d * (series[t] - level) + (Decimal::ONE - gamma_d) * s_prev;
    }

    let next_seasonal = seasonal[series.len() % season_length];
    let forecast = level + trend + next_seasonal;
    Ok(forecast.max(Decimal::ZERO))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn errors_when_history_is_too_short() {
        let series = [dec!(1), dec!(2), dec!(3), dec!(4), dec!(5)];
        let err = holt_winters_additive(&series, 0.3, 0.1, 0.1, 4).unwrap_err();
        assert!(matches!(
            err,
            DemandPlanningError::InsufficientHistory { needed: 8, got: 5 }
        ));
    }

    #[test]
    fn flat_seasonal_pattern_repeats() {
        // 12 weeks of perfectly seasonal pattern (period = 4):
        // weekday peak = 30, weekend trough = 10 — repeated 3 times.
        let pattern = [dec!(30), dec!(20), dec!(15), dec!(10)];
        let mut series: Vec<Decimal> = Vec::new();
        for _ in 0..3 {
            series.extend_from_slice(&pattern);
        }
        let f = holt_winters_additive(&series, 0.3, 0.1, 0.3, 4).unwrap();
        // Forecast for the next bucket should be in the ballpark of 30 (the
        // first seasonal index continues).
        assert!(f >= dec!(15) && f <= dec!(45), "forecast = {}", f);
    }
}
