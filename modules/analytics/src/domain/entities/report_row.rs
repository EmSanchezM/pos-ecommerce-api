//! Report rows — typed result shapes returned by the analytics query repository
//! for each registered `ReportKind`.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakHourRow {
    pub day_of_week: i32, // 0 = Sunday, 6 = Saturday (Postgres EXTRACT(DOW))
    pub hour_of_day: i32, // 0..23
    pub sales_count: i64,
    pub revenue: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductProfitabilityRow {
    pub product_id: Uuid,
    pub product_name: String,
    pub units_sold: Decimal,
    pub revenue: Decimal,
    pub estimated_cost: Decimal,
    pub gross_margin: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadStockRow {
    pub product_id: Uuid,
    pub product_name: String,
    pub last_sold_at: Option<DateTime<Utc>>,
    pub days_since_last_sale: Option<i32>,
    pub current_stock_qty: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashierPerformanceRow {
    pub user_id: Uuid,
    pub user_name: String,
    pub sales_count: i64,
    pub revenue: Decimal,
    pub average_ticket: Decimal,
}

/// Polymorphic envelope returned by `RunReportUseCase`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReportRows {
    PeakHour(Vec<PeakHourRow>),
    ProductProfitability(Vec<ProductProfitabilityRow>),
    DeadStock(Vec<DeadStockRow>),
    CashierPerformance(Vec<CashierPerformanceRow>),
}
