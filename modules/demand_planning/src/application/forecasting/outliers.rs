//! Robust outlier filter — drop points whose modified z-score
//! `|x - median| / (1.4826 * MAD)` exceeds `threshold_sigma`. Default
//! threshold is 3, matching the textbook "three-sigma" cutoff but using
//! median + MAD instead of mean + stddev so a single extreme value can't
//! mask itself by inflating the variance estimate.
//!
//! Series shorter than four points are returned unmodified — too short to
//! estimate scale reliably.
//!
//! Inputs and outputs stay as `Decimal`; intermediate arithmetic is done in
//! `f64` for speed. The MAD constant `1.4826` makes the result consistent
//! with the standard deviation of a normal distribution.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

const MAD_TO_STDDEV: f64 = 1.4826;
const FLOOR_MAD: f64 = 1e-9;

pub fn filter_outliers(series: &[Decimal], threshold_sigma: f64) -> Vec<Decimal> {
    if series.len() < 4 {
        return series.to_vec();
    }
    let mut floats: Vec<f64> = series.iter().filter_map(|d| d.to_f64()).collect();
    if floats.len() < 4 {
        return series.to_vec();
    }
    floats.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = percentile(&floats, 50.0);

    let mut deviations: Vec<f64> = floats.iter().map(|x| (x - median).abs()).collect();
    deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mad = percentile(&deviations, 50.0).max(FLOOR_MAD);

    let robust_sigma = MAD_TO_STDDEV * mad;
    let upper = median + threshold_sigma * robust_sigma;
    let lower = median - threshold_sigma * robust_sigma;

    series
        .iter()
        .copied()
        .filter(|d| {
            let f = d.to_f64().unwrap_or(0.0);
            f >= lower && f <= upper
        })
        .collect()
}

/// Linear-interpolation percentile on a sorted slice.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = (p / 100.0) * (sorted.len() as f64 - 1.0);
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        return sorted[lo];
    }
    let frac = rank - lo as f64;
    sorted[lo] + frac * (sorted[hi] - sorted[lo])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn short_series_returned_as_is() {
        let s = [dec!(1), dec!(100)];
        assert_eq!(filter_outliers(&s, 3.0), s);
    }

    #[test]
    fn drops_extreme_spike() {
        // Stable around 10 with one extreme spike; the robust filter drops it
        // even though the spike alone inflates the naive variance.
        let s = [
            dec!(10),
            dec!(11),
            dec!(9),
            dec!(10),
            dec!(12),
            dec!(8),
            dec!(11),
            dec!(10),
            dec!(500),
        ];
        let filtered = filter_outliers(&s, 3.0);
        assert!(!filtered.contains(&dec!(500)));
    }

    #[test]
    fn constant_series_is_preserved() {
        let s = [dec!(5); 10];
        assert_eq!(filter_outliers(&s, 3.0), s);
    }

    #[test]
    fn keeps_normal_variation() {
        // Modest noise around 100 — nothing should be dropped.
        let s = [
            dec!(98),
            dec!(101),
            dec!(99),
            dec!(102),
            dec!(100),
            dec!(97),
            dec!(103),
            dec!(101),
        ];
        let filtered = filter_outliers(&s, 3.0);
        assert_eq!(filtered.len(), s.len());
    }
}
