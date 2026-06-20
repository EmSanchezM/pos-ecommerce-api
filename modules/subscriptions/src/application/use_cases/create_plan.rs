use std::str::FromStr;
use std::sync::Arc;

use sqlx::{Postgres, Transaction};
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

    /// Non-transactional path (each repo call auto-commits). Used by callers
    /// that don't need to compose the write with anything else.
    pub async fn execute(&self, cmd: CreatePlanCommand) -> Result<PlanResponse, SubscriptionError> {
        let plan = build_plan(cmd)?;
        if self.plan_repo.find_by_code(plan.code()).await?.is_some() {
            return Err(SubscriptionError::CodeAlreadyTaken(
                plan.code().as_str().to_string(),
            ));
        }
        self.plan_repo.save(&plan).await?;
        Ok(PlanResponse::from(plan))
    }

    /// Transactional path — runs the uniqueness check and insert inside the
    /// caller's transaction so the write can commit atomically with, e.g., an
    /// audit-outbox event.
    pub async fn execute_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        cmd: CreatePlanCommand,
    ) -> Result<PlanResponse, SubscriptionError> {
        let plan = build_plan(cmd)?;
        if self
            .plan_repo
            .find_by_code_in_tx(tx, plan.code())
            .await?
            .is_some()
        {
            return Err(SubscriptionError::CodeAlreadyTaken(
                plan.code().as_str().to_string(),
            ));
        }
        self.plan_repo.save_in_tx(tx, &plan).await?;
        Ok(PlanResponse::from(plan))
    }
}

/// Parse + validate + build the plan entity. Pure (no repository access), so it
/// is the single source of the create rules shared by both execution paths.
fn build_plan(cmd: CreatePlanCommand) -> Result<SubscriptionPlan, SubscriptionError> {
    let code = PlanCode::new(cmd.code)?;
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
    Ok(plan)
}
