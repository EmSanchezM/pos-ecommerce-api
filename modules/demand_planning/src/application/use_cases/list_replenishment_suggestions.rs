use std::sync::Arc;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::ReplenishmentSuggestion;
use crate::domain::repositories::ReplenishmentSuggestionRepository;
use crate::domain::value_objects::SuggestionStatus;

pub struct ListReplenishmentSuggestionsUseCase {
    repo: Arc<dyn ReplenishmentSuggestionRepository>,
}

impl ListReplenishmentSuggestionsUseCase {
    pub fn new(repo: Arc<dyn ReplenishmentSuggestionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        store_id: Option<Uuid>,
        status: Option<SuggestionStatus>,
    ) -> Result<Vec<ReplenishmentSuggestion>, DemandPlanningError> {
        self.repo.list(store_id, status).await
    }
}
