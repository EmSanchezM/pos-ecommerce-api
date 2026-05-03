//! RedeemRewardUseCase — checks the member has enough balance, that the
//! reward belongs to the same program, and that the per-member cap (if any)
//! hasn't been hit; then posts a `Redeem` ledger entry and writes a voucher
//! `RewardRedemption` row.

use std::sync::Arc;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::application::dtos::RedeemRewardCommand;
use crate::domain::entities::RewardRedemption;
use crate::domain::repositories::{
    LoyaltyMemberRepository, PointsLedgerRepository, RewardRedemptionRepository, RewardRepository,
};
use crate::domain::value_objects::LoyaltyMemberId;

pub struct RedeemRewardUseCase {
    members: Arc<dyn LoyaltyMemberRepository>,
    rewards: Arc<dyn RewardRepository>,
    ledger: Arc<dyn PointsLedgerRepository>,
    redemptions: Arc<dyn RewardRedemptionRepository>,
}

impl RedeemRewardUseCase {
    pub fn new(
        members: Arc<dyn LoyaltyMemberRepository>,
        rewards: Arc<dyn RewardRepository>,
        ledger: Arc<dyn PointsLedgerRepository>,
        redemptions: Arc<dyn RewardRedemptionRepository>,
    ) -> Self {
        Self {
            members,
            rewards,
            ledger,
            redemptions,
        }
    }

    pub async fn execute(
        &self,
        member_id: LoyaltyMemberId,
        actor_id: Option<Uuid>,
        cmd: RedeemRewardCommand,
    ) -> Result<RewardRedemption, LoyaltyError> {
        let member = self
            .members
            .find_by_id(member_id)
            .await?
            .ok_or_else(|| LoyaltyError::MemberNotFound(member_id.into_uuid()))?;
        let reward = self
            .rewards
            .find_by_id(cmd.reward_id)
            .await?
            .ok_or_else(|| LoyaltyError::RewardNotFound(cmd.reward_id.into_uuid()))?;

        if reward.program_id() != member.program_id() {
            return Err(LoyaltyError::RewardProgramMismatch {
                reward_id: cmd.reward_id.into_uuid(),
                member_id: member_id.into_uuid(),
            });
        }

        if let Some(max) = reward.max_redemptions_per_member() {
            let used = self
                .redemptions
                .count_for_member_reward(member_id, cmd.reward_id)
                .await?;
            if used >= max as i64 {
                return Err(LoyaltyError::RewardProgramMismatch {
                    reward_id: cmd.reward_id.into_uuid(),
                    member_id: member_id.into_uuid(),
                });
            }
        }

        if member.current_balance() < reward.cost_points() {
            return Err(LoyaltyError::InsufficientPoints {
                balance: member.current_balance(),
                required: reward.cost_points(),
            });
        }

        let post = self
            .ledger
            .post_redeem(
                member_id,
                reward.cost_points(),
                Some("reward".into()),
                Some(cmd.reward_id.into_uuid()),
                Some(format!("Redeemed: {}", reward.name())),
                actor_id,
            )
            .await?;

        let redemption = RewardRedemption::create(member_id, cmd.reward_id, post.ledger_entry.id());
        self.redemptions.save(&redemption).await?;
        Ok(redemption)
    }
}
