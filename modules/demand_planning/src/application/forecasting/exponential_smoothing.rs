//! Simple exponential smoothing (single-parameter Holt) — no trend, no
//! seasonality. Closer to recent observations as `alpha → 1`, closer to the
//! long-run mean as `alpha → 0`. We use `alpha = 0.3` by default which is the
//! standard "balanced" choice for retail demand.
//!
//! `S_t = alpha * x_t + (1 - alpha) * S_{t-1}`, seeded with `S_0 = x_0`. The
//! forecast for the next period is the last smoothed value.

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

pub fn exponential_smoothing(series: &[Decimal], alpha: f64) -> Decimal {
    if series.is_empty() {
        return Decimal::ZERO;
    }
    let alpha = alpha.clamp(0.0, 1.0);
    let alpha_d = Decimal::from_f64(alpha).unwrap_or(Decimal::ZERO);
    let one_minus = Decimal::from_f64(1.0 - alpha).unwrap_or(Decimal::ONE);

    let mut s = series[0];
    for x in &series[1..] {
        s = alpha_d * *x + one_minus * s;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn empty_returns_zero() {
        assert_eq!(exponential_smoothing(&[], 0.3), Decimal::ZERO);
    }

    #[test]
    fn constant_series_is_constant() {
        let series = [dec!(50); 8];
        assert_eq!(exponential_smoothing(&series, 0.3), dec!(50));
    }

    #[test]
    fn alpha_zero_yields_first_value() {
        let series = [dec!(10), dec!(20), dec!(30)];
        assert_eq!(exponential_smoothing(&series, 0.0), dec!(10));
    }

    #[test]
    fn alpha_one_yields_last_value() {
        let series = [dec!(10), dec!(20), dec!(30)];
        assert_eq!(exponential_smoothing(&series, 1.0), dec!(30));
    }

    #[test]
    fn responds_to_increasing_trend() {
        let increasing = [dec!(10), dec!(15), dec!(20), dec!(25), dec!(30)];
        let s = exponential_smoothing(&increasing, 0.3);
        // Should sit between the early values and the most recent ones.
        assert!(s > dec!(10));
        assert!(s < dec!(30));
    }
}
