use std::sync::Arc;

use crate::AnalyticsError;
use crate::application::dtos::CreateDashboardCommand;
use crate::domain::entities::Dashboard;
use crate::domain::repositories::DashboardRepository;

pub struct CreateDashboardUseCase {
    repo: Arc<dyn DashboardRepository>,
}

impl CreateDashboardUseCase {
    pub fn new(repo: Arc<dyn DashboardRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, cmd: CreateDashboardCommand) -> Result<Dashboard, AnalyticsError> {
        let dashboard =
            Dashboard::create(cmd.store_id, cmd.owner_user_id, cmd.name, cmd.description);
        self.repo.save(&dashboard).await?;
        Ok(dashboard)
    }
}
