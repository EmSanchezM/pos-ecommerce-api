use std::sync::Arc;

use crate::LoyaltyError;
use crate::domain::entities::LoyaltyMember;
use crate::domain::repositories::LoyaltyMemberRepository;
use crate::domain::value_objects::LoyaltyProgramId;

pub struct ListLoyaltyMembersUseCase {
    repo: Arc<dyn LoyaltyMemberRepository>,
}

impl ListLoyaltyMembersUseCase {
    pub fn new(repo: Arc<dyn LoyaltyMemberRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        program_id: LoyaltyProgramId,
    ) -> Result<Vec<LoyaltyMember>, LoyaltyError> {
        self.repo.list_by_program(program_id).await
    }
}
