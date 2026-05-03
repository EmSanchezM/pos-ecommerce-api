//! EarnPointsUseCase — admin-driven (or v1.1 subscriber-driven) earn flow.
//! Uses the ledger's atomic post operation so concurrent earns from multiple
//! requests don't lose updates. After the post, re-evaluates the member's
//! tier in case the new lifetime crossed a threshold.

use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use super::tier_advancement::maybe_advance_tier;
use crate::LoyaltyError;
use crate::application::dtos::EarnPointsCommand;
use crate::domain::repositories::{
    LoyaltyMemberRepository, LoyaltyProgramRepository, MemberTierRepository,
    PointsLedgerRepository, PostPointsResult,
};
use crate::domain::value_objects::LoyaltyMemberId;

pub struct EarnPointsUseCase {
    programs: Arc<dyn LoyaltyProgramRepository>,
    members: Arc<dyn LoyaltyMemberRepository>,
    tiers: Arc<dyn MemberTierRepository>,
    ledger: Arc<dyn PointsLedgerRepository>,
}

impl EarnPointsUseCase {
    pub fn new(
        programs: Arc<dyn LoyaltyProgramRepository>,
        members: Arc<dyn LoyaltyMemberRepository>,
        tiers: Arc<dyn MemberTierRepository>,
        ledger: Arc<dyn PointsLedgerRepository>,
    ) -> Self {
        Self {
            programs,
            members,
            tiers,
            ledger,
        }
    }

    pub async fn execute(
        &self,
        member_id: LoyaltyMemberId,
        actor_id: Option<Uuid>,
        cmd: EarnPointsCommand,
    ) -> Result<PostPointsResult, LoyaltyError> {
        if cmd.points <= 0 {
            return Err(LoyaltyError::NegativeAmount(cmd.points));
        }

        let member = self
            .members
            .find_by_id(member_id)
            .await?
            .ok_or_else(|| LoyaltyError::MemberNotFound(member_id.into_uuid()))?;
        let program = self
            .programs
            .find_by_id(member.program_id())
            .await?
            .ok_or_else(|| LoyaltyError::ProgramNotFound(member.program_id().into_uuid()))?;

        let expires_at = program
            .expiration_days()
            .map(|days| Utc::now() + Duration::days(days as i64));

        let result = self
            .ledger
            .post_earn(
                member_id,
                cmd.points,
                cmd.source_type,
                cmd.source_id,
                expires_at,
                cmd.reason,
                actor_id,
            )
            .await?;

        // Re-evaluate tier from the just-bumped lifetime total. We rebuild a
        // local LoyaltyMember struct for the helper so we can pass updated
        // figures without a re-read.
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
