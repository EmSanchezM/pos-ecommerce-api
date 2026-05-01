//! KpiKey — stable string identifier for a KPI series.
//!
//! Known KPIs are exposed as associated constants so callers can reference
//! `KpiKey::REVENUE_TOTAL` without typo risk. The type still accepts arbitrary
//! strings so downstream modules (loyalty, accounting) can register their own
//! KPIs without changes here.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KpiKey(String);

impl KpiKey {
    pub const REVENUE_TOTAL: &'static str = "sales.revenue_total";
    pub const SALES_COUNT: &'static str = "sales.count";
    pub const AVERAGE_TICKET: &'static str = "sales.average_ticket";
    pub const UNIQUE_CUSTOMERS: &'static str = "sales.unique_customers";
    pub const STOCK_VALUE: &'static str = "inventory.stock_value";
    pub const LOW_STOCK_COUNT: &'static str = "inventory.low_stock_count";

    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for KpiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for KpiKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for KpiKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}
