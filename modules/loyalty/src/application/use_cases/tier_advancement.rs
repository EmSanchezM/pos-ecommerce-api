//! Shared helper: given a member's lifetime points and the program's tier
//! ladder, returns the highest tier whose threshold is ≤ lifetime. The
//! ladder is expected to be sorted by `threshold_points` ascending — that's
//! the contract the Pg `MemberTierRepository::list_by_program` honors.

use crate::LoyaltyError;
use crate::domain::entities::LoyaltyMember;
use crate::domain::repositories::{LoyaltyMemberRepository, MemberTierRepository};
use crate::domain::value_objects::{LoyaltyProgramId, MemberTierId};

/// Resolves the tier a member should be in given their `lifetime_points`.
/// Returns `None` if no tiers exist or if the member sits below the lowest
/// threshold (which should be 0 for a well-configured program).
pub async fn resolve_tier(
    tiers: &dyn MemberTierRepository,
    program_id: LoyaltyProgramId,
    lifetime_points: i64,
) -> Result<Option<MemberTierId>, LoyaltyError> {
    let ladder = tiers.list_by_program(program_id).await?;
    Ok(ladder
        .iter()
        .rev()
        .find(|t| lifetime_points >= t.threshold_points() && t.is_active())
        .map(|t| t.id()))
}

/// Re-evaluates and persists the tier of a member if it has changed.
pub async fn maybe_advance_tier(
    tiers: &dyn MemberTierRepository,
    members: &dyn LoyaltyMemberRepository,
    member: &LoyaltyMember,
) -> Result<Option<MemberTierId>, LoyaltyError> {
    let resolved = resolve_tier(tiers, member.program_id(), member.lifetime_points()).await?;
    if resolved != member.current_tier_id() {
        members.update_tier(member.id(), resolved).await?;
    }
    Ok(resolved)
}
