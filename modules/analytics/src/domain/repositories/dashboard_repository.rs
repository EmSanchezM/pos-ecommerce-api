use async_trait::async_trait;
use uuid::Uuid;

use crate::AnalyticsError;
use crate::domain::entities::Dashboard;
use crate::domain::value_objects::DashboardId;

#[async_trait]
pub trait DashboardRepository: Send + Sync {
    async fn save(&self, dashboard: &Dashboard) -> Result<(), AnalyticsError>;

    async fn update(&self, dashboard: &Dashboard) -> Result<(), AnalyticsError>;

    async fn find_by_id(&self, id: DashboardId) -> Result<Option<Dashboard>, AnalyticsError>;

    async fn list_for_owner(&self, owner_user_id: Uuid) -> Result<Vec<Dashboard>, AnalyticsError>;

    async fn delete(&self, id: DashboardId) -> Result<(), AnalyticsError>;
}
