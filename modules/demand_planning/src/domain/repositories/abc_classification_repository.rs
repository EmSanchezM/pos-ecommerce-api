use async_trait::async_trait;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::AbcClassification;
use crate::domain::value_objects::AbcClass;

#[async_trait]
pub trait AbcClassificationRepository: Send + Sync {
    async fn save_batch(
        &self,
        classifications: &[AbcClassification],
    ) -> Result<(), DemandPlanningError>;

    async fn find_latest(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Option<AbcClassification>, DemandPlanningError>;

    async fn list(
        &self,
        store_id: Option<Uuid>,
        class: Option<AbcClass>,
    ) -> Result<Vec<AbcClassification>, DemandPlanningError>;
}
