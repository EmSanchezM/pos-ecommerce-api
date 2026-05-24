use async_trait::async_trait;
use sqlx::{Postgres, Transaction};

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

    /// Load an organization inside an existing transaction (for use cases that
    /// need to read + mutate atomically).
    async fn find_by_id_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: OrganizationId,
    ) -> Result<Option<Organization>, TenancyError>;

    /// Persist an updated organization inside an existing transaction.
    async fn update_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        org: &Organization,
    ) -> Result<(), TenancyError>;
}
