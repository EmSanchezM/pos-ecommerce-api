use std::sync::Arc;

use uuid::Uuid;

use crate::AnalyticsError;
use crate::domain::entities::Dashboard;
use crate::domain::repositories::DashboardRepository;

pub struct ListDashboardsUseCase {
    repo: Arc<dyn DashboardRepository>,
}

impl ListDashboardsUseCase {
    pub fn new(repo: Arc<dyn DashboardRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, owner_user_id: Uuid) -> Result<Vec<Dashboard>, AnalyticsError> {
        self.repo.list_for_owner(owner_user_id).await
    }
}
