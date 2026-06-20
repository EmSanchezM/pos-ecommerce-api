use std::sync::Arc;

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::SubscriptionError;
use crate::application::dtos::{PlanResponse, UpdatePlanCommand};
use crate::domain::entities::SubscriptionPlan;
use crate::domain::repositories::SubscriptionPlanRepository;
use crate::domain::value_objects::SubscriptionPlanId;

pub struct UpdatePlanUseCase {
    plan_repo: Arc<dyn SubscriptionPlanRepository>,
}

impl UpdatePlanUseCase {
    pub fn new(plan_repo: Arc<dyn SubscriptionPlanRepository>) -> Self {
        Self { plan_repo }
    }

    /// Non-transactional path.
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
        apply_update(&mut plan, cmd)?;
        self.plan_repo.update(&plan).await?;
        Ok(PlanResponse::from(plan))
    }

    /// Transactional path — load/mutate/persist inside the caller's tx.
    pub async fn execute_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        plan_id: Uuid,
        cmd: UpdatePlanCommand,
    ) -> Result<PlanResponse, SubscriptionError> {
        let id = SubscriptionPlanId::from_uuid(plan_id);
        let mut plan = self
            .plan_repo
            .find_by_id_in_tx(tx, id)
            .await?
            .ok_or(SubscriptionError::PlanNotFound(plan_id))?;
        apply_update(&mut plan, cmd)?;
        self.plan_repo.update_in_tx(tx, &plan).await?;
        Ok(PlanResponse::from(plan))
    }
}

/// Applies the partial update to a loaded plan. Pure (no repository access) —
/// the single source of the update rules shared by both execution paths.
fn apply_update(
    plan: &mut SubscriptionPlan,
    cmd: UpdatePlanCommand,
) -> Result<(), SubscriptionError> {
    // `rename` rewrites both name and description; reuse current values when
    // only one of the two is being touched.
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
    Ok(())
}
