use std::sync::Arc;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::AbcClassification;
use crate::domain::repositories::AbcClassificationRepository;
use crate::domain::value_objects::AbcClass;

pub struct ListAbcClassificationsUseCase {
    repo: Arc<dyn AbcClassificationRepository>,
}

impl ListAbcClassificationsUseCase {
    pub fn new(repo: Arc<dyn AbcClassificationRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        store_id: Option<Uuid>,
        class: Option<AbcClass>,
    ) -> Result<Vec<AbcClassification>, DemandPlanningError> {
        self.repo.list(store_id, class).await
    }
}
