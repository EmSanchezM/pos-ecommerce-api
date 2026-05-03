use async_trait::async_trait;

use crate::TenancyError;
use crate::domain::entities::OrganizationPlan;
use crate::domain::value_objects::OrganizationId;

#[async_trait]
pub trait OrganizationPlanRepository: Send + Sync {
    /// Insert-or-update on (organization_id). One plan per org.
    async fn upsert(&self, plan: &OrganizationPlan) -> Result<(), TenancyError>;
    async fn find_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Option<OrganizationPlan>, TenancyError>;
}
