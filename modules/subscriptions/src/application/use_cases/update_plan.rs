use std::sync::Arc;
use uuid::Uuid;

use crate::SubscriptionError;
use crate::application::dtos::{PlanResponse, UpdatePlanCommand};
use crate::domain::repositories::SubscriptionPlanRepository;
use crate::domain::value_objects::SubscriptionPlanId;

pub struct UpdatePlanUseCase {
    plan_repo: Arc<dyn SubscriptionPlanRepository>,
}

impl UpdatePlanUseCase {
    pub fn new(plan_repo: Arc<dyn SubscriptionPlanRepository>) -> Self {
        Self { plan_repo }
    }

    pub async fn execute(
        &self,
        plan_id: Uuid,
        cmd: UpdatePlanCommand,
    ) -> Result<PlanResponse, SubscriptionError> {
        let id = SubscriptionPlanId::from_uuid(plan_id);
        let mut plan = self
            .plan_repo
            .find_by_id(id)
            .await?
            .ok_or(SubscriptionError::PlanNotFound(plan_id))?;

        // `rename` rewrites both name and description; reuse current values
        // when only one of the two is being touched.
        let name = cmd.name.unwrap_or_else(|| plan.name().to_string());
        let description = match cmd.description {
            Some(value) => value,
            None => plan.description().map(str::to_string),
        };
        plan.rename(name, description);

        if let Some(td) = cmd.trial_days {
            if let Some(d) = td
                && d <= 0
            {
                return Err(SubscriptionError::Validation(
                    "trial_days must be > 0 when set".to_string(),
                ));
            }
            plan.set_trial_days(td);
        }
        if let Some(sort_order) = cmd.sort_order {
            plan.set_sort_order(sort_order);
        }
        if let Some(active) = cmd.is_active {
            if active {
                plan.activate();
            } else {
                plan.deactivate();
            }
        }

        self.plan_repo.update(&plan).await?;
        Ok(PlanResponse::from(plan))
    }
}
