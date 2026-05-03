use async_trait::async_trait;

use crate::TenancyError;
use crate::domain::entities::OrganizationBranding;
use crate::domain::value_objects::OrganizationId;

#[async_trait]
pub trait OrganizationBrandingRepository: Send + Sync {
    /// Insert-or-update on (organization_id). One branding row per org.
    async fn upsert(&self, branding: &OrganizationBranding) -> Result<(), TenancyError>;
    async fn find_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Option<OrganizationBranding>, TenancyError>;
}
