use async_trait::async_trait;

use crate::LoyaltyError;
use crate::domain::entities::MemberTier;
use crate::domain::value_objects::{LoyaltyProgramId, MemberTierId};

#[async_trait]
pub trait MemberTierRepository: Send + Sync {
    async fn save(&self, tier: &MemberTier) -> Result<(), LoyaltyError>;

    async fn find_by_id(&self, id: MemberTierId) -> Result<Option<MemberTier>, LoyaltyError>;

    /// Lists tiers ascending by `threshold_points` so callers can walk the
    /// list to find the highest tier matching a member's lifetime points.
    async fn list_by_program(
        &self,
        program_id: LoyaltyProgramId,
    ) -> Result<Vec<MemberTier>, LoyaltyError>;
}
