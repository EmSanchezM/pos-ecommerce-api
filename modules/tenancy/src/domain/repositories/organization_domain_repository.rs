use async_trait::async_trait;

use crate::TenancyError;
use crate::domain::entities::OrganizationDomain;
use crate::domain::value_objects::{OrganizationDomainId, OrganizationId};

#[async_trait]
pub trait OrganizationDomainRepository: Send + Sync {
    async fn save(&self, domain: &OrganizationDomain) -> Result<(), TenancyError>;
    async fn update(&self, domain: &OrganizationDomain) -> Result<(), TenancyError>;
    async fn delete(&self, id: OrganizationDomainId) -> Result<(), TenancyError>;
    async fn find_by_id(
        &self,
        id: OrganizationDomainId,
    ) -> Result<Option<OrganizationDomain>, TenancyError>;
    async fn find_by_domain(
        &self,
        domain: &str,
    ) -> Result<Option<OrganizationDomain>, TenancyError>;
    async fn list_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationDomain>, TenancyError>;
    /// Atomically clear `is_primary` on every domain of `organization_id`,
    /// then set `is_primary = TRUE` on `target_id`. Used by the
    /// `set_primary_domain` use case to maintain the partial unique index.
    async fn set_primary(
        &self,
        organization_id: OrganizationId,
        target_id: OrganizationDomainId,
    ) -> Result<(), TenancyError>;
}
