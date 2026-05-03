use std::sync::Arc;

use crate::TenancyError;
use crate::application::dtos::RegisterDomainCommand;
use crate::domain::entities::OrganizationDomain;
use crate::domain::repositories::{OrganizationDomainRepository, OrganizationRepository};
use crate::domain::value_objects::{OrganizationDomainId, OrganizationId};

pub struct RegisterDomainUseCase {
    orgs: Arc<dyn OrganizationRepository>,
    domains: Arc<dyn OrganizationDomainRepository>,
}

impl RegisterDomainUseCase {
    pub fn new(
        orgs: Arc<dyn OrganizationRepository>,
        domains: Arc<dyn OrganizationDomainRepository>,
    ) -> Self {
        Self { orgs, domains }
    }

    pub async fn execute(
        &self,
        organization_id: OrganizationId,
        cmd: RegisterDomainCommand,
    ) -> Result<OrganizationDomain, TenancyError> {
        if self.orgs.find_by_id(organization_id).await?.is_none() {
            return Err(TenancyError::OrganizationNotFound(
                organization_id.into_uuid(),
            ));
        }
        // Pre-check the unique constraint so we can return a clean error code
        // instead of letting the DB throw a 23505.
        let domain = OrganizationDomain::register(organization_id, cmd.domain.clone())?;
        if self
            .domains
            .find_by_domain(domain.domain())
            .await?
            .is_some()
        {
            return Err(TenancyError::DomainAlreadyTaken(cmd.domain));
        }
        self.domains.save(&domain).await?;
        Ok(domain)
    }
}

pub struct VerifyDomainUseCase {
    domains: Arc<dyn OrganizationDomainRepository>,
}

impl VerifyDomainUseCase {
    pub fn new(domains: Arc<dyn OrganizationDomainRepository>) -> Self {
        Self { domains }
    }

    pub async fn execute(
        &self,
        id: OrganizationDomainId,
    ) -> Result<OrganizationDomain, TenancyError> {
        let mut domain = self
            .domains
            .find_by_id(id)
            .await?
            .ok_or_else(|| TenancyError::DomainNotFound(id.into_uuid()))?;
        domain.mark_verified();
        self.domains.update(&domain).await?;
        Ok(domain)
    }
}

pub struct SetPrimaryDomainUseCase {
    domains: Arc<dyn OrganizationDomainRepository>,
}

impl SetPrimaryDomainUseCase {
    pub fn new(domains: Arc<dyn OrganizationDomainRepository>) -> Self {
        Self { domains }
    }

    pub async fn execute(
        &self,
        organization_id: OrganizationId,
        target_id: OrganizationDomainId,
    ) -> Result<OrganizationDomain, TenancyError> {
        // Confirm the target belongs to the organization before mutating.
        let target = self
            .domains
            .find_by_id(target_id)
            .await?
            .ok_or_else(|| TenancyError::DomainNotFound(target_id.into_uuid()))?;
        if target.organization_id() != organization_id {
            return Err(TenancyError::Validation(
                "domain does not belong to the supplied organization".to_string(),
            ));
        }
        self.domains.set_primary(organization_id, target_id).await?;
        // Re-fetch for the response (the helper updated rows out-of-band).
        self.domains
            .find_by_id(target_id)
            .await?
            .ok_or_else(|| TenancyError::DomainNotFound(target_id.into_uuid()))
    }
}

pub struct DeleteDomainUseCase {
    domains: Arc<dyn OrganizationDomainRepository>,
}

impl DeleteDomainUseCase {
    pub fn new(domains: Arc<dyn OrganizationDomainRepository>) -> Self {
        Self { domains }
    }

    pub async fn execute(&self, id: OrganizationDomainId) -> Result<(), TenancyError> {
        if self.domains.find_by_id(id).await?.is_none() {
            return Err(TenancyError::DomainNotFound(id.into_uuid()));
        }
        self.domains.delete(id).await
    }
}

pub struct ListDomainsUseCase {
    domains: Arc<dyn OrganizationDomainRepository>,
}

impl ListDomainsUseCase {
    pub fn new(domains: Arc<dyn OrganizationDomainRepository>) -> Self {
        Self { domains }
    }

    pub async fn execute(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationDomain>, TenancyError> {
        self.domains.list_by_organization(organization_id).await
    }
}

pub struct FindOrganizationByDomainUseCase {
    domains: Arc<dyn OrganizationDomainRepository>,
    orgs: Arc<dyn OrganizationRepository>,
}

impl FindOrganizationByDomainUseCase {
    pub fn new(
        domains: Arc<dyn OrganizationDomainRepository>,
        orgs: Arc<dyn OrganizationRepository>,
    ) -> Self {
        Self { domains, orgs }
    }

    pub async fn execute(
        &self,
        host: &str,
    ) -> Result<crate::domain::entities::Organization, TenancyError> {
        let normalised = host.trim().to_ascii_lowercase();
        let domain = self
            .domains
            .find_by_domain(&normalised)
            .await?
            .ok_or_else(|| TenancyError::DomainNotFoundByHostname(normalised.clone()))?;
        self.orgs
            .find_by_id(domain.organization_id())
            .await?
            .ok_or_else(|| TenancyError::OrganizationNotFound(domain.organization_id().into_uuid()))
    }
}
