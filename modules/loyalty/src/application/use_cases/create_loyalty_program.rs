use std::sync::Arc;

use crate::LoyaltyError;
use crate::application::dtos::CreateLoyaltyProgramCommand;
use crate::domain::entities::LoyaltyProgram;
use crate::domain::repositories::LoyaltyProgramRepository;

pub struct CreateLoyaltyProgramUseCase {
    repo: Arc<dyn LoyaltyProgramRepository>,
}

impl CreateLoyaltyProgramUseCase {
    pub fn new(repo: Arc<dyn LoyaltyProgramRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        cmd: CreateLoyaltyProgramCommand,
    ) -> Result<LoyaltyProgram, LoyaltyError> {
        let program = LoyaltyProgram::create(
            cmd.store_id,
            cmd.name,
            cmd.description,
            cmd.points_per_currency_unit,
            cmd.expiration_days,
        );
        self.repo.save(&program).await?;
        Ok(program)
    }
}
