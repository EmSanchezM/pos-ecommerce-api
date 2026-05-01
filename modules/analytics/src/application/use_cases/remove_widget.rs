use std::sync::Arc;

use crate::AnalyticsError;
use crate::domain::repositories::WidgetRepository;
use crate::domain::value_objects::WidgetId;

pub struct RemoveWidgetUseCase {
    repo: Arc<dyn WidgetRepository>,
}

impl RemoveWidgetUseCase {
    pub fn new(repo: Arc<dyn WidgetRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: WidgetId) -> Result<(), AnalyticsError> {
        self.repo.delete(id).await
    }
}
