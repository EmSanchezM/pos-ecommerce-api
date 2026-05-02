use std::sync::Arc;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::application::dtos::DismissSuggestionCommand;
use crate::domain::entities::ReplenishmentSuggestion;
use crate::domain::repositories::ReplenishmentSuggestionRepository;
use crate::domain::value_objects::SuggestionId;

pub struct DismissSuggestionUseCase {
    repo: Arc<dyn ReplenishmentSuggestionRepository>,
}

impl DismissSuggestionUseCase {
    pub fn new(repo: Arc<dyn ReplenishmentSuggestionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        id: SuggestionId,
        actor_id: Uuid,
        cmd: DismissSuggestionCommand,
    ) -> Result<ReplenishmentSuggestion, DemandPlanningError> {
        let mut suggestion = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| DemandPlanningError::SuggestionNotFound(id.into_uuid()))?;
        suggestion.dismiss(actor_id, cmd.reason)?;
        self.repo.update(&suggestion).await?;
        Ok(suggestion)
    }
}
