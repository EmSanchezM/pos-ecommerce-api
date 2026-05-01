mod dashboard;
mod kpi_snapshot;
mod report_row;
mod widget;

pub use dashboard::Dashboard;
pub use kpi_snapshot::KpiSnapshot;
pub use report_row::{
    CashierPerformanceRow, DeadStockRow, PeakHourRow, ProductProfitabilityRow, ReportRows,
};
pub use widget::Widget;
