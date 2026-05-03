use std::sync::Arc;

use crate::LoyaltyError;
use crate::domain::entities::MemberTier;
use crate::domain::repositories::MemberTierRepository;
use crate::domain::value_objects::LoyaltyProgramId;

pub struct ListMemberTiersUseCase {
    repo: Arc<dyn MemberTierRepository>,
}

impl ListMemberTiersUseCase {
    pub fn new(repo: Arc<dyn MemberTierRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        program_id: LoyaltyProgramId,
    ) -> Result<Vec<MemberTier>, LoyaltyError> {
        self.repo.list_by_program(program_id).await
    }
}
