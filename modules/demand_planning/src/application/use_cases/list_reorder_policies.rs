use std::sync::Arc;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::ReorderPolicy;
use crate::domain::repositories::ReorderPolicyRepository;

pub struct ListReorderPoliciesUseCase {
    repo: Arc<dyn ReorderPolicyRepository>,
}

impl ListReorderPoliciesUseCase {
    pub fn new(repo: Arc<dyn ReorderPolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        store_id: Option<Uuid>,
    ) -> Result<Vec<ReorderPolicy>, DemandPlanningError> {
        self.repo.list_active(store_id).await
    }
}
