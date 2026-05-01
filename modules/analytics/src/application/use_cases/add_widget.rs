use std::sync::Arc;

use crate::AnalyticsError;
use crate::application::dtos::AddWidgetCommand;
use crate::domain::entities::Widget;
use crate::domain::repositories::{DashboardRepository, WidgetRepository};
use crate::domain::value_objects::DashboardId;

pub struct AddWidgetUseCase {
    dashboards: Arc<dyn DashboardRepository>,
    widgets: Arc<dyn WidgetRepository>,
}

impl AddWidgetUseCase {
    pub fn new(
        dashboards: Arc<dyn DashboardRepository>,
        widgets: Arc<dyn WidgetRepository>,
    ) -> Self {
        Self {
            dashboards,
            widgets,
        }
    }

    pub async fn execute(
        &self,
        dashboard_id: DashboardId,
        cmd: AddWidgetCommand,
    ) -> Result<Widget, AnalyticsError> {
        // Ensure dashboard exists before attaching widgets.
        let dashboard = self
            .dashboards
            .find_by_id(dashboard_id)
            .await?
            .ok_or_else(|| AnalyticsError::DashboardNotFound(dashboard_id.into_uuid()))?;

        let widget = Widget::create(
            dashboard.id(),
            cmd.title,
            cmd.kind,
            cmd.kpi_key,
            cmd.time_window,
            cmd.position,
            cmd.config,
        );
        self.widgets.save(&widget).await?;
        Ok(widget)
    }
}
