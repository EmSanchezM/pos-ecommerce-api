use async_trait::async_trait;

use crate::TenancyError;
use crate::domain::entities::Organization;
use crate::domain::value_objects::OrganizationId;

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn save(&self, org: &Organization) -> Result<(), TenancyError>;
    async fn update(&self, org: &Organization) -> Result<(), TenancyError>;
    async fn find_by_id(&self, id: OrganizationId) -> Result<Option<Organization>, TenancyError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, TenancyError>;
    async fn list(&self, only_active: bool) -> Result<Vec<Organization>, TenancyError>;
}
