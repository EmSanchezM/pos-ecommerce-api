use std::sync::Arc;
use uuid::Uuid;

use crate::SubscriptionError;
use crate::application::dtos::PlanResponse;
use crate::domain::repositories::SubscriptionPlanRepository;
use crate::domain::value_objects::SubscriptionPlanId;

pub struct GetPlanUseCase {
    plan_repo: Arc<dyn SubscriptionPlanRepository>,
}

impl GetPlanUseCase {
    pub fn new(plan_repo: Arc<dyn SubscriptionPlanRepository>) -> Self {
        Self { plan_repo }
    }

    pub async fn execute(&self, plan_id: Uuid) -> Result<PlanResponse, SubscriptionError> {
        let id = SubscriptionPlanId::from_uuid(plan_id);
        self.plan_repo
            .find_by_id(id)
            .await?
            .map(PlanResponse::from)
            .ok_or(SubscriptionError::PlanNotFound(plan_id))
    }
}
