//! GetDashboardOverviewUseCase — bundles a dashboard with its widgets and the
//! current snapshot for each widget's (kpi_key, time_window). Widgets bound to
//! a `kpi_key` get a snapshot lookup; widgets without one (e.g. report tables)
//! return `None` and the front-end calls `/reports/{kind}/run` separately.

use std::sync::Arc;

use crate::AnalyticsError;
use crate::application::dtos::{
    DashboardOverviewResponse, DashboardResponse, KpiSnapshotResponse, WidgetOverviewResponse,
    WidgetResponse,
};
use crate::domain::repositories::{DashboardRepository, KpiSnapshotRepository, WidgetRepository};
use crate::domain::value_objects::DashboardId;

pub struct GetDashboardOverviewUseCase {
    dashboards: Arc<dyn DashboardRepository>,
    widgets: Arc<dyn WidgetRepository>,
    snapshots: Arc<dyn KpiSnapshotRepository>,
}

impl GetDashboardOverviewUseCase {
    pub fn new(
        dashboards: Arc<dyn DashboardRepository>,
        widgets: Arc<dyn WidgetRepository>,
        snapshots: Arc<dyn KpiSnapshotRepository>,
    ) -> Self {
        Self {
            dashboards,
            widgets,
            snapshots,
        }
    }

    pub async fn execute(
        &self,
        id: DashboardId,
    ) -> Result<DashboardOverviewResponse, AnalyticsError> {
        let dashboard = self
            .dashboards
            .find_by_id(id)
            .await?
            .ok_or_else(|| AnalyticsError::DashboardNotFound(id.into_uuid()))?;

        let widgets = self.widgets.list_by_dashboard(id).await?;

        let mut widget_overviews = Vec::with_capacity(widgets.len());
        for widget in &widgets {
            let snapshot = match widget.kpi_key() {
                Some(key) => self
                    .snapshots
                    .find_latest(key, dashboard.store_id(), widget.time_window())
                    .await?
                    .as_ref()
                    .map(KpiSnapshotResponse::from),
                None => None,
            };
            widget_overviews.push(WidgetOverviewResponse {
                widget: WidgetResponse::from(widget),
                snapshot,
            });
        }

        Ok(DashboardOverviewResponse {
            dashboard: DashboardResponse::from(&dashboard),
            widgets: widget_overviews,
        })
    }
}
