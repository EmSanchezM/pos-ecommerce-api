//! Moving average: `MA_n = mean(series[-n..])`.
//!
//! If the series is shorter than `window`, falls back to the mean of whatever
//! is there. Returns `0` for an empty series so callers can treat it as a safe
//! default.

use rust_decimal::Decimal;

pub fn moving_average(series: &[Decimal], window: usize) -> Decimal {
    if series.is_empty() {
        return Decimal::ZERO;
    }
    let take = window.min(series.len());
    let slice = &series[series.len() - take..];
    let sum: Decimal = slice.iter().copied().sum();
    sum / Decimal::from(take as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn empty_returns_zero() {
        assert_eq!(moving_average(&[], 3), Decimal::ZERO);
    }

    #[test]
    fn shorter_than_window_uses_full_series() {
        let series = [dec!(10), dec!(20)];
        assert_eq!(moving_average(&series, 3), dec!(15));
    }

    #[test]
    fn picks_last_n_elements() {
        let series = [dec!(1), dec!(2), dec!(3), dec!(10), dec!(20), dec!(30)];
        assert_eq!(moving_average(&series, 3), dec!(20));
    }

    #[test]
    fn constant_series_is_idempotent() {
        let series = [dec!(7); 12];
        assert_eq!(moving_average(&series, 3), dec!(7));
        assert_eq!(moving_average(&series, 6), dec!(7));
    }
}
