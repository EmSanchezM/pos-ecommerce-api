use std::sync::Arc;

use crate::LoyaltyError;
use crate::domain::entities::Reward;
use crate::domain::repositories::RewardRepository;
use crate::domain::value_objects::LoyaltyProgramId;

pub struct ListRewardsUseCase {
    repo: Arc<dyn RewardRepository>,
}

impl ListRewardsUseCase {
    pub fn new(repo: Arc<dyn RewardRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, program_id: LoyaltyProgramId) -> Result<Vec<Reward>, LoyaltyError> {
        self.repo.list_by_program(program_id).await
    }
}
