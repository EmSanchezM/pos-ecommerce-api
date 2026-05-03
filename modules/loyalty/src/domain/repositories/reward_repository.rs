use async_trait::async_trait;

use crate::LoyaltyError;
use crate::domain::entities::Reward;
use crate::domain::value_objects::{LoyaltyProgramId, RewardId};

#[async_trait]
pub trait RewardRepository: Send + Sync {
    async fn save(&self, reward: &Reward) -> Result<(), LoyaltyError>;
    async fn find_by_id(&self, id: RewardId) -> Result<Option<Reward>, LoyaltyError>;
    async fn list_by_program(
        &self,
        program_id: LoyaltyProgramId,
    ) -> Result<Vec<Reward>, LoyaltyError>;
}
