use std::sync::Arc;

use tenancy::OrganizationId;

use crate::SubscriptionError;
use crate::application::dtos::{ChangePlanCommand, SubscriptionResponse};
use crate::domain::repositories::{SubscriptionPlanRepository, SubscriptionRepository};
use crate::domain::value_objects::SubscriptionPlanId;

pub struct ChangePlanUseCase {
    plan_repo: Arc<dyn SubscriptionPlanRepository>,
    sub_repo: Arc<dyn SubscriptionRepository>,
}

impl ChangePlanUseCase {
    pub fn new(
        plan_repo: Arc<dyn SubscriptionPlanRepository>,
        sub_repo: Arc<dyn SubscriptionRepository>,
    ) -> Self {
        Self {
            plan_repo,
            sub_repo,
        }
    }

    /// v1.0 semantics: swap `plan_id` immediately. The new pricing kicks in
    /// at the next billing-cycle boundary (the job re-reads the plan when
    /// generating the next cycle). No proration.
    pub async fn execute(
        &self,
        cmd: ChangePlanCommand,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        let org_id = OrganizationId::from_uuid(cmd.organization_id);
        let new_plan_id = SubscriptionPlanId::from_uuid(cmd.new_plan_id);

        let new_plan = self
            .plan_repo
            .find_by_id(new_plan_id)
            .await?
            .ok_or(SubscriptionError::PlanNotFound(cmd.new_plan_id))?;
        if !new_plan.is_active() {
            return Err(SubscriptionError::PlanInactive(cmd.new_plan_id));
        }

        let mut subscription = self
            .sub_repo
            .find_active_by_organization(org_id)
            .await?
            .ok_or(SubscriptionError::SubscriptionNotFound(cmd.organization_id))?;

        subscription.change_plan(new_plan_id);
        self.sub_repo.update_with_version(&subscription).await?;
        Ok(SubscriptionResponse::from(subscription))
    }
}
