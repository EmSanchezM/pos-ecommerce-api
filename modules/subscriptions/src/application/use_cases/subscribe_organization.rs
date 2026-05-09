use std::sync::Arc;

use chrono::Utc;

use tenancy::OrganizationId;

use crate::SubscriptionError;
use crate::application::dtos::{SubscribeOrganizationCommand, SubscriptionResponse};
use crate::domain::entities::{BillingCycle, Subscription};
use crate::domain::repositories::{
    BillingCycleRepository, SubscriptionPlanRepository, SubscriptionRepository,
};
use crate::domain::value_objects::{BillingCycleStatus, SubscriptionPlanId, SubscriptionStatus};

pub struct SubscribeOrganizationUseCase {
    plan_repo: Arc<dyn SubscriptionPlanRepository>,
    sub_repo: Arc<dyn SubscriptionRepository>,
    cycle_repo: Arc<dyn BillingCycleRepository>,
}

impl SubscribeOrganizationUseCase {
    pub fn new(
        plan_repo: Arc<dyn SubscriptionPlanRepository>,
        sub_repo: Arc<dyn SubscriptionRepository>,
        cycle_repo: Arc<dyn BillingCycleRepository>,
    ) -> Self {
        Self {
            plan_repo,
            sub_repo,
            cycle_repo,
        }
    }

    pub async fn execute(
        &self,
        cmd: SubscribeOrganizationCommand,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        let org_id = OrganizationId::from_uuid(cmd.organization_id);
        let plan_id = SubscriptionPlanId::from_uuid(cmd.plan_id);

        if self
            .sub_repo
            .find_active_by_organization(org_id)
            .await?
            .is_some()
        {
            return Err(SubscriptionError::OrganizationAlreadySubscribed(
                cmd.organization_id,
            ));
        }

        let plan = self
            .plan_repo
            .find_by_id(plan_id)
            .await?
            .ok_or(SubscriptionError::PlanNotFound(cmd.plan_id))?;
        if !plan.is_active() {
            return Err(SubscriptionError::PlanInactive(cmd.plan_id));
        }

        let now = Utc::now();
        let subscription = Subscription::start(&plan, org_id, now);
        self.sub_repo.save(&subscription).await?;

        // Bootstrap the first billing cycle. Trial → Trialing/0 amount;
        // otherwise a Pending row that the billing job will pick up.
        let initial_status = match subscription.status() {
            SubscriptionStatus::Trialing => BillingCycleStatus::Trialing,
            _ => BillingCycleStatus::Pending,
        };
        let amount_cents = if matches!(subscription.status(), SubscriptionStatus::Trialing) {
            0
        } else {
            plan.price_cents()
        };
        let cycle = BillingCycle::create(
            subscription.id(),
            subscription.current_period_start(),
            subscription.current_period_end(),
            amount_cents,
            plan.currency().to_string(),
            initial_status,
        );
        self.cycle_repo.save(&cycle).await?;

        // TODO(events): publish `subscription.created` once we wire a
        // transactional event publisher into this module.
        Ok(SubscriptionResponse::from(subscription))
    }
}
