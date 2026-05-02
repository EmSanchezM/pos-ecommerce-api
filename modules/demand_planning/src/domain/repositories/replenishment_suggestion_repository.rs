use async_trait::async_trait;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::ReplenishmentSuggestion;
use crate::domain::value_objects::{SuggestionId, SuggestionStatus};

#[async_trait]
pub trait ReplenishmentSuggestionRepository: Send + Sync {
    async fn save(&self, suggestion: &ReplenishmentSuggestion) -> Result<(), DemandPlanningError>;

    /// Persist a status transition + linked PO id (when applicable).
    async fn update(&self, suggestion: &ReplenishmentSuggestion)
    -> Result<(), DemandPlanningError>;

    async fn find_by_id(
        &self,
        id: SuggestionId,
    ) -> Result<Option<ReplenishmentSuggestion>, DemandPlanningError>;

    async fn list(
        &self,
        store_id: Option<Uuid>,
        status: Option<SuggestionStatus>,
    ) -> Result<Vec<ReplenishmentSuggestion>, DemandPlanningError>;

    /// Returns true if there is already a pending suggestion for this
    /// (variant, store), so the generator can skip duplicates.
    async fn has_pending_for(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<bool, DemandPlanningError>;
}
