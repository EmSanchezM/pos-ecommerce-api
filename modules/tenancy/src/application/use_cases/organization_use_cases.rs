use std::sync::Arc;

use crate::TenancyError;
use crate::application::dtos::{RegisterOrganizationCommand, UpdateOrganizationCommand};
use crate::domain::entities::{Organization, OrganizationPlan};
use crate::domain::repositories::{OrganizationPlanRepository, OrganizationRepository};
use crate::domain::value_objects::{OrganizationId, PlanTier};

pub struct RegisterOrganizationUseCase {
    orgs: Arc<dyn OrganizationRepository>,
    plans: Arc<dyn OrganizationPlanRepository>,
}

impl RegisterOrganizationUseCase {
    pub fn new(
        orgs: Arc<dyn OrganizationRepository>,
        plans: Arc<dyn OrganizationPlanRepository>,
    ) -> Self {
        Self { orgs, plans }
    }

    pub async fn execute(
        &self,
        cmd: RegisterOrganizationCommand,
    ) -> Result<(Organization, OrganizationPlan), TenancyError> {
        if self.orgs.find_by_slug(&cmd.slug).await?.is_some() {
            return Err(TenancyError::SlugAlreadyTaken(cmd.slug));
        }
        let org = Organization::register(cmd.name, cmd.slug, cmd.contact_email, cmd.contact_phone)?;
        self.orgs.save(&org).await?;

        let tier = cmd.initial_tier.unwrap_or(PlanTier::Free);
        let plan = OrganizationPlan::new(org.id(), tier, None, None, None, None)?;
        self.plans.upsert(&plan).await?;
        Ok((org, plan))
    }
}

pub struct UpdateOrganizationUseCase {
    orgs: Arc<dyn OrganizationRepository>,
}

impl UpdateOrganizationUseCase {
    pub fn new(orgs: Arc<dyn OrganizationRepository>) -> Self {
        Self { orgs }
    }

    pub async fn execute(
        &self,
        id: OrganizationId,
        cmd: UpdateOrganizationCommand,
    ) -> Result<Organization, TenancyError> {
        let mut org = self
            .orgs
            .find_by_id(id)
            .await?
            .ok_or_else(|| TenancyError::OrganizationNotFound(id.into_uuid()))?;
        org.update_contact(cmd.name, cmd.contact_email, cmd.contact_phone)?;
        self.orgs.update(&org).await?;
        Ok(org)
    }
}

pub struct SuspendOrganizationUseCase {
    orgs: Arc<dyn OrganizationRepository>,
}

impl SuspendOrganizationUseCase {
    pub fn new(orgs: Arc<dyn OrganizationRepository>) -> Self {
        Self { orgs }
    }

    pub async fn execute(&self, id: OrganizationId) -> Result<Organization, TenancyError> {
        let mut org = self
            .orgs
            .find_by_id(id)
            .await?
            .ok_or_else(|| TenancyError::OrganizationNotFound(id.into_uuid()))?;
        org.suspend()?;
        self.orgs.update(&org).await?;
        Ok(org)
    }
}

pub struct ActivateOrganizationUseCase {
    orgs: Arc<dyn OrganizationRepository>,
}

impl ActivateOrganizationUseCase {
    pub fn new(orgs: Arc<dyn OrganizationRepository>) -> Self {
        Self { orgs }
    }

    pub async fn execute(&self, id: OrganizationId) -> Result<Organization, TenancyError> {
        let mut org = self
            .orgs
            .find_by_id(id)
            .await?
            .ok_or_else(|| TenancyError::OrganizationNotFound(id.into_uuid()))?;
        org.activate()?;
        self.orgs.update(&org).await?;
        Ok(org)
    }
}

pub struct GetOrganizationUseCase {
    orgs: Arc<dyn OrganizationRepository>,
}

impl GetOrganizationUseCase {
    pub fn new(orgs: Arc<dyn OrganizationRepository>) -> Self {
        Self { orgs }
    }

    pub async fn execute(&self, id: OrganizationId) -> Result<Organization, TenancyError> {
        self.orgs
            .find_by_id(id)
            .await?
            .ok_or_else(|| TenancyError::OrganizationNotFound(id.into_uuid()))
    }
}

pub struct GetOrganizationBySlugUseCase {
    orgs: Arc<dyn OrganizationRepository>,
}

impl GetOrganizationBySlugUseCase {
    pub fn new(orgs: Arc<dyn OrganizationRepository>) -> Self {
        Self { orgs }
    }

    pub async fn execute(&self, slug: &str) -> Result<Organization, TenancyError> {
        self.orgs
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| TenancyError::OrganizationNotFoundBySlug(slug.to_string()))
    }
}

pub struct ListOrganizationsUseCase {
    orgs: Arc<dyn OrganizationRepository>,
}

impl ListOrganizationsUseCase {
    pub fn new(orgs: Arc<dyn OrganizationRepository>) -> Self {
        Self { orgs }
    }

    pub async fn execute(&self, only_active: bool) -> Result<Vec<Organization>, TenancyError> {
        self.orgs.list(only_active).await
    }
}
