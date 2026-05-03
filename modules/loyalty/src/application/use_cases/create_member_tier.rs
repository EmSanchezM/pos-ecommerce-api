use std::sync::Arc;

use crate::LoyaltyError;
use crate::application::dtos::CreateMemberTierCommand;
use crate::domain::entities::MemberTier;
use crate::domain::repositories::{LoyaltyProgramRepository, MemberTierRepository};

pub struct CreateMemberTierUseCase {
    programs: Arc<dyn LoyaltyProgramRepository>,
    tiers: Arc<dyn MemberTierRepository>,
}

impl CreateMemberTierUseCase {
    pub fn new(
        programs: Arc<dyn LoyaltyProgramRepository>,
        tiers: Arc<dyn MemberTierRepository>,
    ) -> Self {
        Self { programs, tiers }
    }

    pub async fn execute(&self, cmd: CreateMemberTierCommand) -> Result<MemberTier, LoyaltyError> {
        // Validate program exists.
        self.programs
            .find_by_id(cmd.program_id)
            .await?
            .ok_or_else(|| LoyaltyError::ProgramNotFound(cmd.program_id.into_uuid()))?;
        let tier = MemberTier::create(
            cmd.program_id,
            cmd.name,
            cmd.threshold_points,
            cmd.benefits,
            cmd.sort_order,
        )?;
        self.tiers.save(&tier).await?;
        Ok(tier)
    }
}
