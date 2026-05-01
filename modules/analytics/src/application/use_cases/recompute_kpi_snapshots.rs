//! RecomputeKpiSnapshotsUseCase — refreshes the canonical KPI snapshots used
//! by dashboards. Run periodically by `api-gateway::jobs::analytics_recompute`.
//!
//! For now the canonical set is hardcoded (revenue, sales count, average ticket,
//! unique customers across Today / ThisWeek / ThisMonth, with `store_id = None`
//! meaning system-wide). Per-store recomputes can be added by passing a list of
//! store ids when scaling out.

use std::sync::Arc;

use chrono::Utc;
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;

use crate::AnalyticsError;
use crate::domain::entities::KpiSnapshot;
use crate::domain::repositories::{AnalyticsQueryRepository, KpiSnapshotRepository};
use crate::domain::value_objects::{KpiKey, TimeWindow};

pub struct RecomputeKpiSnapshotsUseCase {
    queries: Arc<dyn AnalyticsQueryRepository>,
    snapshots: Arc<dyn KpiSnapshotRepository>,
}

const WINDOWS: &[TimeWindow] = &[
    TimeWindow::Today,
    TimeWindow::ThisWeek,
    TimeWindow::ThisMonth,
];

impl RecomputeKpiSnapshotsUseCase {
    pub fn new(
        queries: Arc<dyn AnalyticsQueryRepository>,
        snapshots: Arc<dyn KpiSnapshotRepository>,
    ) -> Self {
        Self { queries, snapshots }
    }

    /// Recomputes the canonical KPIs for the given stores. Pass `&[None]` for a
    /// single system-wide pass; pass `&[Some(store_a), Some(store_b)]` for a
    /// per-store refresh.
    ///
    /// Returns the number of snapshots written.
    pub async fn execute(&self, stores: &[Option<Uuid>]) -> Result<usize, AnalyticsError> {
        let now = Utc::now();
        let mut written = 0usize;

        for store_id in stores {
            for window in WINDOWS {
                let (from, to) = window.bounds(now);

                let revenue = self.queries.total_revenue(*store_id, from, to).await?;
                self.persist(KpiKey::REVENUE_TOTAL, *store_id, *window, revenue)
                    .await?;
                written += 1;

                let count = self.queries.sales_count(*store_id, from, to).await?;
                self.persist(
                    KpiKey::SALES_COUNT,
                    *store_id,
                    *window,
                    Decimal::from(count),
                )
                .await?;
                written += 1;

                let avg_ticket = if count > 0 {
                    revenue / Decimal::from(count)
                } else {
                    Decimal::ZERO
                };
                self.persist(KpiKey::AVERAGE_TICKET, *store_id, *window, avg_ticket)
                    .await?;
                written += 1;

                let customers = self.queries.unique_customers(*store_id, from, to).await?;
                self.persist(
                    KpiKey::UNIQUE_CUSTOMERS,
                    *store_id,
                    *window,
                    Decimal::from(customers),
                )
                .await?;
                written += 1;
            }
        }

        Ok(written)
    }

    async fn persist(
        &self,
        key: &str,
        store_id: Option<Uuid>,
        window: TimeWindow,
        value: Decimal,
    ) -> Result<(), AnalyticsError> {
        let snapshot = KpiSnapshot::create(KpiKey::new(key), store_id, window, value, json!({}));
        self.snapshots.upsert(&snapshot).await
    }
}
