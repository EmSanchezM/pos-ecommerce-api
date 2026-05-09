use std::sync::Arc;
use uuid::Uuid;

use crate::SubscriptionError;
use crate::domain::repositories::SubscriptionPlanRepository;
use crate::domain::value_objects::SubscriptionPlanId;

pub struct DeactivatePlanUseCase {
    plan_repo: Arc<dyn SubscriptionPlanRepository>,
}

impl DeactivatePlanUseCase {
    pub fn new(plan_repo: Arc<dyn SubscriptionPlanRepository>) -> Self {
        Self { plan_repo }
    }

    pub async fn execute(&self, plan_id: Uuid) -> Result<(), SubscriptionError> {
        let id = SubscriptionPlanId::from_uuid(plan_id);
        let mut plan = self
            .plan_repo
            .find_by_id(id)
            .await?
            .ok_or(SubscriptionError::PlanNotFound(plan_id))?;
        plan.deactivate();
        self.plan_repo.update(&plan).await?;
        Ok(())
    }
}
