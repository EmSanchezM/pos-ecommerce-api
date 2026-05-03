use async_trait::async_trait;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::RewardRedemption;
use crate::domain::value_objects::{LoyaltyMemberId, RewardId, RewardRedemptionId};

#[async_trait]
pub trait RewardRedemptionRepository: Send + Sync {
    async fn save(&self, redemption: &RewardRedemption) -> Result<(), LoyaltyError>;

    async fn find_by_id(
        &self,
        id: RewardRedemptionId,
    ) -> Result<Option<RewardRedemption>, LoyaltyError>;

    async fn count_for_member_reward(
        &self,
        member_id: LoyaltyMemberId,
        reward_id: RewardId,
    ) -> Result<i64, LoyaltyError>;

    async fn list_for_member(
        &self,
        member_id: LoyaltyMemberId,
    ) -> Result<Vec<RewardRedemption>, LoyaltyError>;

    /// Marks the voucher as applied to a sale. Called by the storefront once
    /// the discount is consumed.
    async fn mark_applied(&self, id: RewardRedemptionId, sale_id: Uuid)
    -> Result<(), LoyaltyError>;
}
