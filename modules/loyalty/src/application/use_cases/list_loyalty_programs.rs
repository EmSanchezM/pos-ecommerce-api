use std::sync::Arc;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::LoyaltyProgram;
use crate::domain::repositories::LoyaltyProgramRepository;

pub struct ListLoyaltyProgramsUseCase {
    repo: Arc<dyn LoyaltyProgramRepository>,
}

impl ListLoyaltyProgramsUseCase {
    pub fn new(repo: Arc<dyn LoyaltyProgramRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        store_id: Option<Uuid>,
    ) -> Result<Vec<LoyaltyProgram>, LoyaltyError> {
        self.repo.list(store_id).await
    }
}
