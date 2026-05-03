//! AdjustPointsUseCase — admin correction. Signed delta. Re-evaluates the
//! member's tier afterwards in case lifetime crossed a threshold.

use std::sync::Arc;
use uuid::Uuid;

use super::tier_advancement::maybe_advance_tier;
use crate::LoyaltyError;
use crate::application::dtos::AdjustPointsCommand;
use crate::domain::repositories::{
    LoyaltyMemberRepository, MemberTierRepository, PointsLedgerRepository, PostPointsResult,
};
use crate::domain::value_objects::LoyaltyMemberId;

pub struct AdjustPointsUseCase {
    members: Arc<dyn LoyaltyMemberRepository>,
    tiers: Arc<dyn MemberTierRepository>,
    ledger: Arc<dyn PointsLedgerRepository>,
}

impl AdjustPointsUseCase {
    pub fn new(
        members: Arc<dyn LoyaltyMemberRepository>,
        tiers: Arc<dyn MemberTierRepository>,
        ledger: Arc<dyn PointsLedgerRepository>,
    ) -> Self {
        Self {
            members,
            tiers,
            ledger,
        }
    }

    pub async fn execute(
        &self,
        member_id: LoyaltyMemberId,
        actor_id: Uuid,
        cmd: AdjustPointsCommand,
    ) -> Result<PostPointsResult, LoyaltyError> {
        if cmd.points == 0 {
            return Err(LoyaltyError::NegativeAmount(0));
        }

        let member = self
            .members
            .find_by_id(member_id)
            .await?
            .ok_or_else(|| LoyaltyError::MemberNotFound(member_id.into_uuid()))?;

        let result = self
            .ledger
            .post_adjustment(member_id, cmd.points, cmd.reason, actor_id)
            .await?;

        let updated = crate::domain::entities::LoyaltyMember::reconstitute(
            member.id(),
            member.program_id(),
            member.customer_id(),
            member.current_tier_id(),
            result.current_balance,
            result.lifetime_points,
            member.enrolled_at(),
            member.updated_at(),
        );
        maybe_advance_tier(self.tiers.as_ref(), self.members.as_ref(), &updated).await?;

        Ok(result)
    }
}
