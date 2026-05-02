//! SeriesPoint — a single bucket of historical demand: how many units of a
//! product variant were sold in a given period (day, week, or month). Read
//! exclusively from the existing `sales` data via `SalesHistoryRepository`;
//! the demand_planning module does not own the storage.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesPoint {
    pub period_start: NaiveDate,
    pub units: Decimal,
}

impl SeriesPoint {
    pub fn new(period_start: NaiveDate, units: Decimal) -> Self {
        Self {
            period_start,
            units,
        }
    }
}
