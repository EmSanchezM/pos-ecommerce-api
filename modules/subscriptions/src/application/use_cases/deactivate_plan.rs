use std::sync::Arc;

use sqlx::{Postgres, Transaction};
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

    /// Non-transactional path.
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

    /// Transactional path — load/deactivate/persist inside the caller's tx.
    pub async fn execute_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        plan_id: Uuid,
    ) -> Result<(), SubscriptionError> {
        let id = SubscriptionPlanId::from_uuid(plan_id);
        let mut plan = self
            .plan_repo
            .find_by_id_in_tx(tx, id)
            .await?
            .ok_or(SubscriptionError::PlanNotFound(plan_id))?;
        plan.deactivate();
        self.plan_repo.update_in_tx(tx, &plan).await?;
        Ok(())
    }
}
