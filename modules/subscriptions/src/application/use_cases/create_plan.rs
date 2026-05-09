use std::str::FromStr;
use std::sync::Arc;

use tenancy::PlanTier;

use crate::SubscriptionError;
use crate::application::dtos::{CreatePlanCommand, PlanResponse};
use crate::domain::entities::SubscriptionPlan;
use crate::domain::repositories::SubscriptionPlanRepository;
use crate::domain::value_objects::{BillingInterval, PlanCode};

pub struct CreatePlanUseCase {
    plan_repo: Arc<dyn SubscriptionPlanRepository>,
}

impl CreatePlanUseCase {
    pub fn new(plan_repo: Arc<dyn SubscriptionPlanRepository>) -> Self {
        Self { plan_repo }
    }

    pub async fn execute(&self, cmd: CreatePlanCommand) -> Result<PlanResponse, SubscriptionError> {
        let code = PlanCode::new(cmd.code)?;

        if self.plan_repo.find_by_code(&code).await?.is_some() {
            return Err(SubscriptionError::CodeAlreadyTaken(
                code.as_str().to_string(),
            ));
        }

        let tier = PlanTier::from_str(&cmd.tier)
            .map_err(|e| SubscriptionError::Validation(format!("invalid tier: {e}")))?;
        let interval = BillingInterval::from_str(&cmd.interval)?;

        if cmd.price_cents < 0 {
            return Err(SubscriptionError::Validation(
                "price_cents must be >= 0".to_string(),
            ));
        }
        if cmd.currency.len() != 3 {
            return Err(SubscriptionError::Validation(
                "currency must be a 3-letter ISO-4217 code".to_string(),
            ));
        }
        if let Some(d) = cmd.trial_days
            && d <= 0
        {
            return Err(SubscriptionError::Validation(
                "trial_days must be > 0 when set".to_string(),
            ));
        }

        let mut plan = SubscriptionPlan::create(
            code,
            cmd.name,
            tier,
            interval,
            cmd.price_cents,
            cmd.currency.to_uppercase(),
        );
        if let Some(desc) = cmd.description {
            plan.rename(plan.name().to_string(), Some(desc));
        }
        if cmd.sort_order != 0 {
            plan.set_sort_order(cmd.sort_order);
        }
        if let Some(td) = cmd.trial_days {
            plan.set_trial_days(Some(td));
        }

        self.plan_repo.save(&plan).await?;
        Ok(PlanResponse::from(plan))
    }
}
