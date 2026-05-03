use async_trait::async_trait;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::LoyaltyMember;
use crate::domain::value_objects::{LoyaltyMemberId, LoyaltyProgramId, MemberTierId};

#[async_trait]
pub trait LoyaltyMemberRepository: Send + Sync {
    async fn save(&self, member: &LoyaltyMember) -> Result<(), LoyaltyError>;

    async fn find_by_id(&self, id: LoyaltyMemberId) -> Result<Option<LoyaltyMember>, LoyaltyError>;

    async fn find_by_customer(
        &self,
        program_id: LoyaltyProgramId,
        customer_id: Uuid,
    ) -> Result<Option<LoyaltyMember>, LoyaltyError>;

    async fn list_by_program(
        &self,
        program_id: LoyaltyProgramId,
    ) -> Result<Vec<LoyaltyMember>, LoyaltyError>;

    /// Update only the cached tier — used after a tier-advancement check.
    async fn update_tier(
        &self,
        id: LoyaltyMemberId,
        tier_id: Option<MemberTierId>,
    ) -> Result<(), LoyaltyError>;
}
