use std::sync::Arc;

use crate::TenancyError;
use crate::application::dtos::{SetFeatureFlagCommand, SetPlanCommand};
use crate::domain::entities::OrganizationPlan;
use crate::domain::repositories::{OrganizationPlanRepository, OrganizationRepository};
use crate::domain::value_objects::OrganizationId;

pub struct SetPlanUseCase {
    orgs: Arc<dyn OrganizationRepository>,
    plans: Arc<dyn OrganizationPlanRepository>,
}

impl SetPlanUseCase {
    pub fn new(
        orgs: Arc<dyn OrganizationRepository>,
        plans: Arc<dyn OrganizationPlanRepository>,
    ) -> Self {
        Self { orgs, plans }
    }

    pub async fn execute(
        &self,
        organization_id: OrganizationId,
        cmd: SetPlanCommand,
    ) -> Result<OrganizationPlan, TenancyError> {
        if self.orgs.find_by_id(organization_id).await?.is_none() {
            return Err(TenancyError::OrganizationNotFound(
                organization_id.into_uuid(),
            ));
        }
        let existing = self.plans.find_by_organization(organization_id).await?;
        let plan = match existing {
            Some(mut existing) => {
                // Caller wins if they sent flags explicitly. Otherwise: a
                // tier change snaps to the new tier's defaults (admin
                // intent), a same-tier upsert keeps the current overrides.
                let flags = cmd.feature_flags.unwrap_or_else(|| {
                    if existing.tier() != cmd.tier {
                        cmd.tier.default_feature_flags()
                    } else {
                        existing.feature_flags().clone()
                    }
                });
                existing.update(
                    cmd.tier,
                    flags,
                    cmd.seat_limit,
                    cmd.store_limit,
                    cmd.expires_at,
                )?;
                existing
            }
            None => OrganizationPlan::new(
                organization_id,
                cmd.tier,
                cmd.feature_flags,
                cmd.seat_limit,
                cmd.store_limit,
                cmd.expires_at,
            )?,
        };
        self.plans.upsert(&plan).await?;
        Ok(plan)
    }
}

pub struct SetFeatureFlagUseCase {
    plans: Arc<dyn OrganizationPlanRepository>,
}

impl SetFeatureFlagUseCase {
    pub fn new(plans: Arc<dyn OrganizationPlanRepository>) -> Self {
        Self { plans }
    }

    pub async fn execute(
        &self,
        organization_id: OrganizationId,
        cmd: SetFeatureFlagCommand,
    ) -> Result<OrganizationPlan, TenancyError> {
        let mut plan = self
            .plans
            .find_by_organization(organization_id)
            .await?
            .ok_or_else(|| TenancyError::PlanNotFound(organization_id.into_uuid()))?;
        plan.set_feature(&cmd.feature, cmd.enabled);
        self.plans.upsert(&plan).await?;
        Ok(plan)
    }
}

pub struct GetPlanUseCase {
    plans: Arc<dyn OrganizationPlanRepository>,
}

impl GetPlanUseCase {
    pub fn new(plans: Arc<dyn OrganizationPlanRepository>) -> Self {
        Self { plans }
    }

    pub async fn execute(
        &self,
        organization_id: OrganizationId,
    ) -> Result<OrganizationPlan, TenancyError> {
        self.plans
            .find_by_organization(organization_id)
            .await?
            .ok_or_else(|| TenancyError::PlanNotFound(organization_id.into_uuid()))
    }
}
