use std::sync::Arc;

use crate::LoyaltyError;
use crate::application::dtos::CreateRewardCommand;
use crate::domain::entities::Reward;
use crate::domain::repositories::{LoyaltyProgramRepository, RewardRepository};

pub struct CreateRewardUseCase {
    programs: Arc<dyn LoyaltyProgramRepository>,
    rewards: Arc<dyn RewardRepository>,
}

impl CreateRewardUseCase {
    pub fn new(
        programs: Arc<dyn LoyaltyProgramRepository>,
        rewards: Arc<dyn RewardRepository>,
    ) -> Self {
        Self { programs, rewards }
    }

    pub async fn execute(&self, cmd: CreateRewardCommand) -> Result<Reward, LoyaltyError> {
        self.programs
            .find_by_id(cmd.program_id)
            .await?
            .ok_or_else(|| LoyaltyError::ProgramNotFound(cmd.program_id.into_uuid()))?;
        let reward = Reward::create(
            cmd.program_id,
            cmd.name,
            cmd.description,
            cmd.cost_points,
            cmd.reward_type,
            cmd.reward_value,
            cmd.max_redemptions_per_member,
        )?;
        self.rewards.save(&reward).await?;
        Ok(reward)
    }
}
