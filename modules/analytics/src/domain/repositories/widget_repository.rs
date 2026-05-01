use async_trait::async_trait;

use crate::AnalyticsError;
use crate::domain::entities::Widget;
use crate::domain::value_objects::{DashboardId, WidgetId};

#[async_trait]
pub trait WidgetRepository: Send + Sync {
    async fn save(&self, widget: &Widget) -> Result<(), AnalyticsError>;

    async fn find_by_id(&self, id: WidgetId) -> Result<Option<Widget>, AnalyticsError>;

    async fn list_by_dashboard(
        &self,
        dashboard_id: DashboardId,
    ) -> Result<Vec<Widget>, AnalyticsError>;

    async fn delete(&self, id: WidgetId) -> Result<(), AnalyticsError>;
}
