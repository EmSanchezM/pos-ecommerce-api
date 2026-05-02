use async_trait::async_trait;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::ReorderPolicy;
use crate::domain::value_objects::ReorderPolicyId;

#[async_trait]
pub trait ReorderPolicyRepository: Send + Sync {
    async fn save(&self, policy: &ReorderPolicy) -> Result<(), DemandPlanningError>;

    /// Update with optimistic locking against `version`.
    async fn update(&self, policy: &ReorderPolicy) -> Result<(), DemandPlanningError>;

    async fn find_by_id(
        &self,
        id: ReorderPolicyId,
    ) -> Result<Option<ReorderPolicy>, DemandPlanningError>;

    async fn find_by_variant_store(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Option<ReorderPolicy>, DemandPlanningError>;

    async fn list_active(
        &self,
        store_id: Option<Uuid>,
    ) -> Result<Vec<ReorderPolicy>, DemandPlanningError>;
}
