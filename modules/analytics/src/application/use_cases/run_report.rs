//! RunReportUseCase — dispatches a `ReportKind` to the corresponding query on
//! `AnalyticsQueryRepository` and wraps the result in `ReportRows`.

use std::sync::Arc;

use chrono::Utc;

use crate::AnalyticsError;
use crate::application::dtos::RunReportCommand;
use crate::domain::entities::ReportRows;
use crate::domain::repositories::{AnalyticsQueryRepository, ReportFilters};
use crate::domain::value_objects::ReportKind;

pub struct RunReportUseCase {
    queries: Arc<dyn AnalyticsQueryRepository>,
}

impl RunReportUseCase {
    pub fn new(queries: Arc<dyn AnalyticsQueryRepository>) -> Self {
        Self { queries }
    }

    pub async fn execute(&self, cmd: RunReportCommand) -> Result<ReportRows, AnalyticsError> {
        let (from, to) = cmd.time_window.bounds(Utc::now());
        let filters = ReportFilters {
            store_id: cmd.store_id,
            from,
            to,
            limit: cmd.limit.clamp(1, 1000),
        };

        let rows = match cmd.kind {
            ReportKind::PeakHour => ReportRows::PeakHour(self.queries.peak_hour(&filters).await?),
            ReportKind::ProductProfitability => ReportRows::ProductProfitability(
                self.queries.product_profitability(&filters).await?,
            ),
            ReportKind::DeadStock => {
                ReportRows::DeadStock(self.queries.dead_stock(&filters).await?)
            }
            ReportKind::CashierPerformance => {
                ReportRows::CashierPerformance(self.queries.cashier_performance(&filters).await?)
            }
        };

        Ok(rows)
    }
}
