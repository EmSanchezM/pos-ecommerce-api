use std::sync::Arc;

use crate::LoyaltyError;
use crate::domain::entities::LoyaltyMember;
use crate::domain::repositories::LoyaltyMemberRepository;
use crate::domain::value_objects::LoyaltyMemberId;

pub struct GetLoyaltyMemberUseCase {
    repo: Arc<dyn LoyaltyMemberRepository>,
}

impl GetLoyaltyMemberUseCase {
    pub fn new(repo: Arc<dyn LoyaltyMemberRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: LoyaltyMemberId) -> Result<LoyaltyMember, LoyaltyError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| LoyaltyError::MemberNotFound(id.into_uuid()))
    }
}
