//! ReportKind — registered cross-module reports the analytics module can run.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::AnalyticsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportKind {
    /// Sales count + revenue grouped by hour-of-day and day-of-week.
    PeakHour,
    /// Per-product revenue, units sold, gross margin.
    ProductProfitability,
    /// Products with no sales in the configured window — capital trapped.
    DeadStock,
    /// Sales count, revenue, and average ticket per cashier.
    CashierPerformance,
}

impl fmt::Display for ReportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ReportKind::PeakHour => "peak_hour",
            ReportKind::ProductProfitability => "product_profitability",
            ReportKind::DeadStock => "dead_stock",
            ReportKind::CashierPerformance => "cashier_performance",
        };
        f.write_str(s)
    }
}

impl FromStr for ReportKind {
    type Err = AnalyticsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "peak_hour" => Ok(Self::PeakHour),
            "product_profitability" => Ok(Self::ProductProfitability),
            "dead_stock" => Ok(Self::DeadStock),
            "cashier_performance" => Ok(Self::CashierPerformance),
            other => Err(AnalyticsError::UnknownReportKind(other.into())),
        }
    }
}
